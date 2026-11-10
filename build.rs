// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Generates the typed `res::` constants for `resource/` (https://daybrite.dev/docs/how-to/resources)
//! and the table of fixture web pages under `tests/fixtures/web/` that fixture mode serves
//! (src/fixtures.rs).

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    day_build::prebuild_project().expect("day-build: prebuild");
    fixture_table();
}

fn fixture_table() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let root = manifest.join("tests/fixtures/web");
    println!("cargo:rerun-if-changed=tests/fixtures/web");
    let mut files = Vec::new();
    collect(&root, &root, &mut files);
    files.sort();
    let mut out = String::from(
        "/// The fixture site files, by `<host>/<path>`.\npub static FILES: &[(&str, &[u8])] = &[\n",
    );
    for (rel, abs) in files {
        let _ = writeln!(out, "    ({rel:?}, include_bytes!({abs:?})),");
    }
    out.push_str("];\n");
    let dest = PathBuf::from(std::env::var("OUT_DIR").expect("out dir")).join("web_fixtures.rs");
    std::fs::write(dest, out).expect("write the fixture table");
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, out);
        } else if let Ok(rel) = path.strip_prefix(root) {
            let rel = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            if !rel.starts_with('.') && !rel.ends_with(".md") {
                out.push((rel, path.to_string_lossy().into_owned()));
            }
        }
    }
}
