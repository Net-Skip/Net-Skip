// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! Shared test harness: a scratch cache directory, a clock the test moves, a simulated list
//! server with ETags, and a blocker wired to all three.

#![allow(dead_code)] // each test file uses part of the harness

use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use day_part_http::simulation::{Reply, Simulation};
use day_part_http::{Client, Session};
use netskip_adblock::{AdBlock, Clock, Level, ListFormat, Purpose, Settings, Source, SourceId};

pub const ADS: &str = include_str!("../data/ads.txt");
pub const PRIVACY: &str = include_str!("../data/privacy.txt");
pub const HOSTS: &str = include_str!("../data/hosts.txt");

pub const ADS_URL: &str = "https://lists.test/ads.txt";
pub const PRIVACY_URL: &str = "https://lists.test/privacy.txt";
pub const HOSTS_URL: &str = "https://lists.test/hosts.txt";

/// Run a future to completion on this thread, parking between polls.
pub fn block_on<F: Future>(future: F) -> F::Output {
    struct Unpark(std::thread::Thread);
    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        std::thread::park_timeout(Duration::from_millis(50));
    }
}

/// A fresh, empty directory under the system temp directory.
pub fn scratch_dir(name: &str) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "netskip-adblock-tests/{name}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// A clock that only moves when the test says so.
#[derive(Clone)]
pub struct FakeClock(Arc<Mutex<SystemTime>>);

impl FakeClock {
    pub fn new() -> FakeClock {
        FakeClock(Arc::new(Mutex::new(
            UNIX_EPOCH + Duration::from_secs(1_800_000_000),
        )))
    }
    pub fn advance(&self, by: Duration) {
        *self.0.lock().unwrap() += by;
    }
    pub fn secs(&self) -> u64 {
        self.now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

impl Clock for FakeClock {
    fn now(&self) -> SystemTime {
        *self.0.lock().unwrap()
    }
}

/// How the simulated server answers one URL.
#[derive(Clone)]
pub enum Serve {
    /// The body, with an ETag derived from it; a matching `If-None-Match` gets a 304.
    Body(String),
    /// This status with an empty body.
    Status(u16),
}

/// Every request the list server saw: (URL, `If-None-Match` header).
pub type RequestLog = Arc<Mutex<Vec<(String, Option<String>)>>>;

/// A simulated list server whose answers a test can change between requests.
#[derive(Clone)]
pub struct ListServer {
    pub simulation: Simulation,
    pub session: Session,
    routes: Arc<Mutex<HashMap<String, Serve>>>,
    /// Every request: (URL, If-None-Match header).
    pub log: RequestLog,
}

impl ListServer {
    pub fn new() -> ListServer {
        let simulation = Simulation::new();
        let routes: Arc<Mutex<HashMap<String, Serve>>> = Arc::default();
        let log: RequestLog = Arc::default();
        let (r, l) = (routes.clone(), log.clone());
        simulation.handle(
            |_, url| url.starts_with("https://lists.test/"),
            move |request| {
                let inm = request
                    .headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("if-none-match"))
                    .map(|(_, v)| v.clone());
                l.lock().unwrap().push((request.url.clone(), inm.clone()));
                let serve = r.lock().unwrap().get(&request.url).cloned();
                Ok(match serve {
                    Some(Serve::Body(body)) => {
                        let etag = format!("\"{:x}\"", body.len() * 31 + body.lines().count());
                        if inm.as_deref() == Some(etag.as_str()) {
                            Reply::new(304, Vec::new()).header("ETag", etag)
                        } else {
                            Reply::new(200, body.into_bytes())
                                .header("ETag", etag)
                                .header("Content-Type", "text/plain")
                        }
                    }
                    Some(Serve::Status(status)) => Reply::new(status, Vec::new()),
                    None => Reply::new(404, Vec::new()),
                })
            },
        );
        let session = Session::new();
        session.set_simulation(Some(simulation.clone()));
        ListServer {
            simulation,
            session,
            routes,
            log,
        }
    }

    pub fn serve(&self, url: &str, serve: Serve) {
        self.routes.lock().unwrap().insert(url.to_string(), serve);
    }

    pub fn requests_to(&self, url: &str) -> Vec<Option<String>> {
        self.log
            .lock()
            .unwrap()
            .iter()
            .filter(|(u, _)| u == url)
            .map(|(_, inm)| inm.clone())
            .collect()
    }

    pub fn client(&self) -> Client {
        Client::builder().session(self.session.clone()).build()
    }

    /// Serve the three fixture lists.
    pub fn with_fixtures() -> ListServer {
        let server = ListServer::new();
        server.serve(ADS_URL, Serve::Body(ADS.into()));
        server.serve(PRIVACY_URL, Serve::Body(PRIVACY.into()));
        server.serve(HOSTS_URL, Serve::Body(HOSTS.into()));
        server
    }
}

pub fn ads_source() -> Source {
    Source::custom(ADS_URL)
        .with_title("Day test ads")
        .with_purpose(Purpose::Ads)
}

pub fn privacy_source() -> Source {
    Source::custom(PRIVACY_URL)
        .with_title("Day test privacy")
        .with_purpose(Purpose::Privacy)
}

pub fn hosts_source() -> Source {
    Source::custom(HOSTS_URL)
        .with_title("Day test hosts")
        .with_purpose(Purpose::Security)
        .with_format(ListFormat::Hosts)
}

/// A custom level over the fixture lists.
pub fn fixture_level(document_blocking: bool, same_site_blocking: bool) -> Level {
    Level::Custom {
        sources: vec![ads_source().id, privacy_source().id, hosts_source().id],
        document_blocking,
        same_site_blocking,
    }
}

pub fn fixture_settings(level: Level) -> Settings {
    Settings {
        level,
        custom_sources: vec![ads_source(), privacy_source(), hosts_source()],
        ..Settings::default()
    }
}

pub fn blocker(
    dir: &std::path::Path,
    server: &ListServer,
    clock: &FakeClock,
    level: Level,
) -> AdBlock {
    AdBlock::builder()
        .cache_dir(dir)
        .client(server.client())
        .clock(Arc::new(clock.clone()))
        .settings(fixture_settings(level))
        .build()
        .expect("blocker")
}

/// A blocker over the fixture lists, downloaded and compiled.
pub fn ready_blocker(level: Level) -> (AdBlock, ListServer, FakeClock, PathBuf) {
    let dir = scratch_dir("ready");
    let server = ListServer::with_fixtures();
    let clock = FakeClock::new();
    let adblock = blocker(&dir, &server, &clock, level);
    let report = block_on(adblock.update_now());
    assert!(report.recompiled, "{report:?}");
    (adblock, server, clock, dir)
}

pub fn ids(list: &[SourceId]) -> Vec<&str> {
    list.iter().map(SourceId::as_str).collect()
}
