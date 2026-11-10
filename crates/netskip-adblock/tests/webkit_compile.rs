// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! Hands the exported content rules to WebKit's own compiler, `WKContentRuleListStore`, on
//! macOS. A custom harness, because the store reports on the main thread and only `main`
//! can run the main run loop. Other platforms build an empty `main`.
//!
//! With `DAY_ADBLOCK_LISTS=<dir>` set, the real lists in that directory (`easylist.txt`,
//! `easyprivacy.txt`, ...) are exported and compiled too.

#[cfg(target_os = "macos")]
mod common;

#[cfg(not(target_os = "macos"))]
fn main() {}

#[cfg(target_os = "macos")]
fn main() {
    use common::*;

    let level = fixture_level(false, false);
    let (adblock, ..) = ready_blocker(level.clone());
    let rules = block_on(adblock.content_rules(&level)).expect("export");
    let mut failures = 0;
    let store_dir = scratch_dir("wk-store");
    for list in &rules.lists {
        let json = list.read_json().expect("rules file");
        match webkit::compile(&store_dir, &list.identifier, &json) {
            Ok(()) => println!("fixture {}: {} rules compiled", list.identifier, list.rules),
            Err(e) => {
                failures += 1;
                println!("fixture {}: {e}", list.identifier);
            }
        }
    }
    // WebKit's own parser must refuse a broken list, or the check above proves nothing.
    match webkit::compile(
        &store_dir,
        "broken",
        r#"[{"trigger":{"url-filter":"a|b"},"action":{"type":"block"}}]"#,
    ) {
        Ok(()) => {
            failures += 1;
            println!("WebKit accepted a disjunction; the check is not reaching the compiler");
        }
        Err(e) => println!("a broken list is refused: {e}"),
    }

    if let Ok(dir) = std::env::var("DAY_ADBLOCK_LISTS") {
        failures += real_lists(std::path::Path::new(&dir));
    }
    if failures > 0 {
        eprintln!("{failures} content rule list(s) failed to compile");
        std::process::exit(1);
    }
    println!("test result: ok");
}

#[cfg(target_os = "macos")]
fn real_lists(dir: &std::path::Path) -> usize {
    use common::*;
    use netskip_adblock::{AdBlock, Level, Purpose, Settings, Source};
    use std::time::Instant;

    // The real lists go through the normal download path, served by the simulated server.
    let server = ListServer::new();
    let mut sources = Vec::new();
    for (file, purpose) in [
        ("easylist.txt", Purpose::Ads),
        ("easyprivacy.txt", Purpose::Privacy),
    ] {
        if let Ok(text) = std::fs::read_to_string(dir.join(file)) {
            let url = format!("https://lists.test/{file}");
            server.serve(&url, Serve::Body(text));
            sources.push(Source::custom(&url).with_purpose(purpose));
        }
    }
    if sources.is_empty() {
        println!("real lists: none found in {}", dir.display());
        return 0;
    }
    let level = Level::Custom {
        sources: sources.iter().map(|s| s.id.clone()).collect(),
        document_blocking: false,
        same_site_blocking: false,
    };
    let adblock = AdBlock::builder()
        .cache_dir(scratch_dir("wk-real"))
        .client(server.client())
        .settings(Settings {
            level: level.clone(),
            custom_sources: sources,
            ..Settings::default()
        })
        .build()
        .expect("blocker");
    block_on(adblock.update_now());
    let started = Instant::now();
    let rules = match block_on(adblock.content_rules(&level)) {
        Ok(rules) => rules,
        Err(e) => {
            println!("real lists: export failed: {e}");
            return 1;
        }
    };
    println!("real lists: exported in {:?}", started.elapsed());
    let store_dir = scratch_dir("wk-real-store");
    let mut failures = 0;
    for list in &rules.lists {
        let json = list.read_json().expect("rules file");
        let started = Instant::now();
        match webkit::compile(&store_dir, &list.identifier, &json) {
            Ok(()) => println!(
                "real {}: {} rules, {} bytes, compiled in {:?}",
                list.identifier,
                list.rules,
                json.len(),
                started.elapsed()
            ),
            Err(e) => {
                failures += 1;
                println!("real {}: {e}", list.identifier);
            }
        }
    }
    failures
}

#[cfg(target_os = "macos")]
mod webkit {
    use std::cell::RefCell;
    use std::ffi::{CStr, CString};
    use std::path::Path;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use block2::RcBlock;
    use objc2::rc::{Retained, autoreleasepool};
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};

    #[link(name = "WebKit", kind = "framework")]
    unsafe extern "C" {}
    #[link(name = "Foundation", kind = "framework")]
    unsafe extern "C" {}

    fn ns_string(s: &str) -> Retained<AnyObject> {
        let c = CString::new(s).expect("no NUL in rules");
        // SAFETY: `stringWithUTF8String:` copies a valid NUL-terminated UTF-8 C string.
        let out: Option<Retained<AnyObject>> =
            unsafe { msg_send![class!(NSString), stringWithUTF8String: c.as_ptr()] };
        out.expect("NSString")
    }

    fn describe(error: *mut AnyObject) -> String {
        if error.is_null() {
            return "unknown error".into();
        }
        // SAFETY: `error` is the NSError WebKit passed to the completion handler, alive for
        // the duration of the handler; `localizedDescription` returns an NSString whose
        // UTF8String stays valid while that string lives.
        unsafe {
            let text: *mut AnyObject = msg_send![error, localizedDescription];
            let utf8: *const std::ffi::c_char = msg_send![text, UTF8String];
            CStr::from_ptr(utf8).to_string_lossy().into_owned()
        }
    }

    /// Compile one rule list into a store under `dir`, running the main run loop until WebKit
    /// answers.
    pub fn compile(dir: &Path, identifier: &str, json: &str) -> Result<(), String> {
        autoreleasepool(|_| {
            let path = ns_string(&dir.to_string_lossy());
            // SAFETY: plain Foundation and WebKit class methods with object arguments of the
            // documented types; every returned object is retained before use.
            let store: Retained<AnyObject> = unsafe {
                let url: Option<Retained<AnyObject>> =
                    msg_send![class!(NSURL), fileURLWithPath: &*path, isDirectory: true];
                let url = url.ok_or("NSURL")?;
                let store: Option<Retained<AnyObject>> =
                    msg_send![class!(WKContentRuleListStore), storeWithURL: &*url];
                store.ok_or("WKContentRuleListStore")?
            };
            let result: Rc<RefCell<Option<Result<(), String>>>> = Rc::default();
            let slot = result.clone();
            let handler = RcBlock::new(move |list: *mut AnyObject, error: *mut AnyObject| {
                *slot.borrow_mut() = Some(if !list.is_null() {
                    Ok(())
                } else {
                    Err(describe(error))
                });
            });
            let identifier = ns_string(identifier);
            let encoded = ns_string(json);
            // SAFETY: the documented selector with an NSString identifier, an NSString list,
            // and a `void (^)(WKContentRuleList *, NSError *)` handler the store copies.
            unsafe {
                let _: () = msg_send![
                    &*store,
                    compileContentRuleListForIdentifier: &*identifier,
                    encodedContentRuleList: &*encoded,
                    completionHandler: &*handler
                ];
            }
            let deadline = Instant::now() + Duration::from_secs(300);
            while result.borrow().is_none() {
                if Instant::now() > deadline {
                    return Err("WebKit did not answer within five minutes".into());
                }
                // SAFETY: runs the current (main) thread's run loop for 50 ms.
                unsafe {
                    let until: Retained<AnyObject> =
                        msg_send![class!(NSDate), dateWithTimeIntervalSinceNow: 0.05f64];
                    let run_loop: *mut AnyObject = msg_send![class!(NSRunLoop), currentRunLoop];
                    let _: () = msg_send![run_loop, runUntilDate: &*until];
                }
            }
            result
                .borrow_mut()
                .take()
                .unwrap_or(Err("no result".into()))
        })
    }
}
