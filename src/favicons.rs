// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Site icons: read from the page's `<link rel=icon>` (or `/favicon.ico`), scaled to 64 pixels,
//! kept as PNG per site in memory and in the app's cache directory.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use day::prelude::*;
use netskip_model::address;

use crate::browser::{Icon, Tab};
use crate::fixtures;

/// The size icons are stored at, in pixels: sharp at 32 points on a 2× screen.
const SIZE: u32 = 64;

thread_local! {
    static CACHE: RefCell<HashMap<String, Option<Icon>>> = RefCell::new(HashMap::new());
    /// Bumped when an icon arrives, so views showing one re-read the cache.
    static VERSION: Signal<u64> = Signal::global(0);
}

/// The icon for a page's site, if one is known. A tracked read: it changes when one arrives.
pub fn for_url(url: &str) -> Option<Icon> {
    VERSION.with(|v| v.get());
    let site = address::site(url)?;
    if let Some(hit) = CACHE.with(|c| c.borrow().get(&site).cloned()) {
        return hit;
    }
    let disk = read_disk(&site).map(|bytes| Icon(Rc::from(bytes)));
    CACHE.with(|c| c.borrow_mut().insert(site, disk.clone()));
    disk
}

fn cache_path(site: &str) -> Option<std::path::PathBuf> {
    let safe: String = site
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    Some(
        day::storage::cache_dir()?
            .join("favicons")
            .join(format!("{safe}.png")),
    )
}

fn read_disk(site: &str) -> Option<Vec<u8>> {
    std::fs::read(cache_path(site)?).ok()
}

fn write_disk(site: &str, png: &[u8]) {
    if let Some(path) = cache_path(site) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, png);
    }
}

/// The icons a page lists, best first: a touch icon or the largest sized icon, then the rest,
/// then `/favicon.ico` on the page's origin.
pub fn candidates(page_url: &str, info_json: &str) -> Vec<String> {
    let info: serde_json::Value = serde_json::from_str(info_json).unwrap_or_default();
    let mut icons: Vec<(i64, String)> = info
        .get("icons")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|i| {
                    let href = fixtures::to_logical(i.get("href")?.as_str()?);
                    let rel = i.get("rel").and_then(|v| v.as_str()).unwrap_or_default();
                    let ty = i.get("type").and_then(|v| v.as_str()).unwrap_or_default();
                    let sizes = i.get("sizes").and_then(|v| v.as_str()).unwrap_or_default();
                    if ty.contains("svg") || href.ends_with(".svg") {
                        // Scaled once and stored as PNG: vector icons need a renderer this
                        // decoder does not have.
                        return None;
                    }
                    let size = sizes
                        .split_whitespace()
                        .filter_map(|s| s.split(['x', 'X']).next()?.parse::<i64>().ok())
                        .max()
                        .unwrap_or(if rel.contains("apple-touch") { 180 } else { 32 });
                    // Prefer the size closest above the stored size.
                    let score = if size >= i64::from(SIZE) {
                        1000 - size
                    } else {
                        size
                    };
                    Some((score, href))
                })
                .collect()
        })
        .unwrap_or_default();
    icons.sort_by_key(|i| std::cmp::Reverse(i.0));
    let mut out: Vec<String> = icons.into_iter().map(|(_, h)| h).collect();
    if let Ok(url) = url::Url::parse(page_url)
        && matches!(url.scheme(), "http" | "https")
    {
        out.push(format!(
            "{}/favicon.ico",
            url.origin().ascii_serialization()
        ));
    }
    out.dedup();
    out
}

/// Scale an icon image to [`SIZE`] and encode it as PNG.
pub fn normalize(bytes: &[u8]) -> Option<Vec<u8>> {
    let img = image::load_from_memory(bytes).ok()?;
    let img = if img.width() == SIZE && img.height() == SIZE {
        img
    } else {
        img.resize_exact(SIZE, SIZE, image::imageops::FilterType::Lanczos3)
    };
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

/// After a page loads: show its site's icon in the tab, fetching it when it is not cached.
pub fn load(tab: Tab, page_url: &str, info_json: &str) {
    let Some(site) = address::site(page_url) else {
        tab.icon.set(None);
        return;
    };
    if let Some(icon) = untrack(|| for_url(page_url)) {
        tab.icon.set(Some(icon));
        return;
    }
    let urls = candidates(page_url, info_json);
    day::task(async move {
        for url in urls {
            let request =
                day_part_http::Request::get(&url).timeout(std::time::Duration::from_secs(8));
            let Ok(response) = day_part_http::fetch_future(request).await else {
                continue;
            };
            if response.status != 200 || response.body.is_empty() {
                continue;
            }
            let Some(png) = normalize(&response.body) else {
                continue;
            };
            write_disk(&site, &png);
            let icon = Icon(Rc::from(png));
            CACHE.with(|c| c.borrow_mut().insert(site.clone(), Some(icon.clone())));
            VERSION.with(|v| v.update(|n| *n += 1));
            tab.icon.set(Some(icon));
            return;
        }
    });
}

/// Forget every stored icon (Clear Website Data).
pub fn clear() {
    CACHE.with(|c| c.borrow_mut().clear());
    if let Some(dir) = day::storage::cache_dir() {
        let _ = std::fs::remove_dir_all(dir.join("favicons"));
    }
    VERSION.with(|v| v.update(|n| *n += 1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_prefer_large_raster_icons_then_favicon_ico() {
        let info = r#"{"icons":[
            {"href":"https://a.example/16.png","rel":"icon","sizes":"16x16","type":"image/png"},
            {"href":"https://a.example/i.svg","rel":"icon","sizes":"any","type":"image/svg+xml"},
            {"href":"https://a.example/touch.png","rel":"apple-touch-icon","sizes":"","type":""},
            {"href":"https://a.example/64.png","rel":"icon","sizes":"64x64","type":"image/png"}
        ]}"#;
        assert_eq!(
            candidates("https://a.example/page", info),
            [
                "https://a.example/64.png",
                "https://a.example/touch.png",
                "https://a.example/16.png",
                "https://a.example/favicon.ico"
            ]
        );
        assert_eq!(
            candidates("https://b.example/", "garbage"),
            ["https://b.example/favicon.ico"]
        );
        assert!(candidates("about:blank", "{}").is_empty());
    }

    #[test]
    fn icons_are_scaled_to_png() {
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(16, 16, image::Rgba([10, 20, 30, 255]))
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let out = normalize(png.get_ref()).unwrap();
        let back = image::load_from_memory(&out).unwrap();
        assert_eq!((back.width(), back.height()), (SIZE, SIZE));
        assert!(normalize(b"not an image").is_none());
    }
}
