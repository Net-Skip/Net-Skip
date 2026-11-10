// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! The browser's library on disk: history and bookmarks, in one SQLite file through
//! day-persistence (https://daybrite.dev/docs/reference/persistence).
//!
//! One [`DatabaseWorker`] owns the file for the whole process, so a history that grows to tens
//! of thousands of pages never makes typing in the address bar wait on the disk: the UI asks
//! through [`read`] and [`write`] and awaits owned values. The functions below take the
//! worker's [`ModelContainer`], so the tests call them directly on an in-memory container.
//! Every write ends in an explicit `save`, which inside a worker write is part of that write's
//! one transaction.

use std::future::Future;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll, Wake, Waker};

use day::prelude::*;
use day_persistence::prelude::*;
use day_persistence::{DatabaseWorker, DbError, ModelContainer, Sqlite, Value, WorkerRequest};
use netskip_model::address;
use netskip_model::complete::Candidate;

/// The library file under the app's data directory.
const FILE: &str = "library.db";

/// Rows the worker keeps resident per model; reads return owned values, so little is held.
const CACHE_ROWS: usize = 2048;

/// How many history pages completion considers for one keystroke.
pub const CANDIDATE_LIMIT: usize = 300;

/// One page in history: the latest title, and how often and how recently it was visited.
#[derive(Model, Clone, Default, PartialEq, Debug)]
#[model(table = "history")]
pub struct HistoryPage {
    #[model(id)]
    pub id: u64,
    /// The page's address without its fragment ([`address::history_key`]).
    #[model(unique)]
    pub url: String,
    pub title: String,
    pub visits: i64,
    /// Visits that started from the address bar.
    pub typed: i64,
    /// Milliseconds since the Unix epoch.
    #[model(index)]
    pub last_visit: i64,
    pub first_visit: i64,
}

/// A saved page. Bookmarks are one ordered list; the start page shows them as Favorites.
#[derive(Model, Clone, Default, PartialEq, Debug)]
#[model(table = "bookmarks")]
pub struct Bookmark {
    #[model(id)]
    pub id: u64,
    #[model(index)]
    pub url: String,
    pub title: String,
    #[model(index)]
    pub position: i64,
    pub added: i64,
}

// --- the worker ----------------------------------------------------------------------------

static WORKER: OnceLock<DatabaseWorker> = OnceLock::new();

/// The process's library worker, opened on first use. `DAY_DATA_DIR` (which dayscripts set)
/// moves the file with the rest of the app's data; `NETSKIP_MEMORY_LIBRARY=1` keeps it in
/// memory, for test runs that must start empty.
pub fn worker() -> &'static DatabaseWorker {
    WORKER.get_or_init(|| {
        let memory = day::env("NETSKIP_MEMORY_LIBRARY").as_deref() == Some("1");
        let dir = day::storage::data_dir();
        let opened = match (memory, dir) {
            (false, Some(dir)) => {
                let path = dir.join(FILE);
                block_on(DatabaseWorker::open(move || {
                    std::fs::create_dir_all(&dir)
                        .map_err(|e| DbError::driver(format!("creating {}: {e}", dir.display())))?;
                    open(Sqlite::at(&path))
                }))
            }
            _ => block_on(DatabaseWorker::open(|| open(Sqlite::memory()))),
        };
        match opened {
            Ok(worker) => worker,
            Err(e) => {
                error!("the library could not open ({e}); this session keeps it in memory");
                // An in-memory SQLite that cannot open is a broken build, not a runtime state.
                block_on(DatabaseWorker::open(|| open(Sqlite::memory())))
                    .expect("in-memory library")
            }
        }
    })
}

/// Run a read on the library worker.
pub fn read<R: Send + 'static>(
    f: impl FnOnce(&ModelContainer) -> Result<R, DbError> + Send + 'static,
) -> WorkerRequest<R> {
    worker().read(f)
}

/// Run one transaction on the library worker. It has committed when the request completes.
pub fn write<R: Send + 'static>(
    f: impl FnOnce(&ModelContainer) -> Result<R, DbError> + Send + 'static,
) -> WorkerRequest<R> {
    worker().write(f)
}

/// Wait for a future on a thread with no executor (opening the worker). Never on the UI thread
/// for anything slower than opening.
pub fn block_on<F: Future>(f: F) -> F::Output {
    struct Unpark(std::thread::Thread);
    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut f = std::pin::pin!(f);
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::park();
    }
}

/// Open the library in `driver`.
pub fn open(driver: Sqlite) -> Result<ModelContainer, DbError> {
    let db = ModelContainer::open(driver, schema![HistoryPage, Bookmark])?;
    db.set_cache_limit(CACHE_ROWS);
    Ok(db)
}

fn next_id<M: Model>(db: &ModelContainer, table: &str) -> Result<u64, DbError> {
    let last = db
        .query_raw::<M>(
            format!("SELECT id FROM {table} ORDER BY id DESC LIMIT 1"),
            Vec::new(),
            &[table],
        )
        .try_ids()?;
    Ok(last.first().map(|id| id.handle() + 1).unwrap_or(1))
}

// --- history -------------------------------------------------------------------------------

fn page_by_url(db: &ModelContainer, url: &str) -> Result<Option<HistoryPage>, DbError> {
    Ok(db
        .query::<HistoryPage>()
        .filter(HistoryPage::url().eq(url.to_string()))
        .limit(1)
        .live()
        .try_collect()?
        .into_iter()
        .next())
}

/// Record a visit to `url` (already a history key) at `now`. A repeat visit counts up and moves
/// the page to the top; a title replaces the stored one only when there is one.
pub fn record_visit(
    db: &ModelContainer,
    url: &str,
    title: Option<&str>,
    typed: bool,
    now: i64,
) -> Result<u64, DbError> {
    let id = match page_by_url(db, url)? {
        Some(page) => {
            if let Some(row) = db.try_get::<HistoryPage>(page.id)? {
                row.visits().write(page.visits + 1);
                if typed {
                    row.typed().write(page.typed + 1);
                }
                row.last_visit().write(now);
                if let Some(t) = title
                    && t != page.title
                {
                    row.title().write(t.to_string());
                }
            }
            page.id
        }
        None => {
            let id = next_id::<HistoryPage>(db, "history")?;
            db.insert(HistoryPage {
                id,
                url: url.to_string(),
                title: title.unwrap_or_default().to_string(),
                visits: 1,
                typed: i64::from(typed),
                last_visit: now,
                first_visit: now,
            });
            id
        }
    };
    db.save()?;
    Ok(id)
}

/// Update a visited page's title, which often arrives after the load that recorded it.
pub fn set_title(db: &ModelContainer, url: &str, title: &str) -> Result<(), DbError> {
    let title = title.trim();
    if title.is_empty() {
        return Ok(());
    }
    if let Some(page) = page_by_url(db, url)?
        && page.title != title
        && let Some(row) = db.try_get::<HistoryPage>(page.id)?
    {
        row.title().write(title.to_string());
    }
    db.save()
}

/// History newest first, optionally only pages whose title or address holds `search` (case
/// folded), at most `limit` pages.
pub fn history(
    db: &ModelContainer,
    search: &str,
    limit: usize,
) -> Result<Vec<HistoryPage>, DbError> {
    let q = search.trim().to_lowercase();
    let mut sql = String::from("SELECT id FROM history");
    let mut params = Vec::new();
    if !q.is_empty() {
        sql.push_str(" WHERE instr(day_fold(url), ?) > 0 OR instr(day_fold(title), ?) > 0");
        params.extend(std::iter::repeat_n(Value::Text(q), 2));
    }
    sql.push_str(&format!(" ORDER BY last_visit DESC, id DESC LIMIT {limit}"));
    db.query_raw::<HistoryPage>(sql, params, &["history"])
        .try_collect()
}

/// Remove these pages from history.
pub fn delete_pages(db: &ModelContainer, ids: &[u64]) -> Result<(), DbError> {
    for id in ids {
        db.delete::<HistoryPage>(*id)?;
    }
    db.save()
}

/// Remove every page last visited at or after `since` (everything, for `None`). Returns how
/// many pages went.
pub fn clear_history(db: &ModelContainer, since: Option<i64>) -> Result<usize, DbError> {
    let ids = match since {
        Some(since) => db
            .query::<HistoryPage>()
            .filter(HistoryPage::last_visit().ge(since))
            .live()
            .try_ids()?,
        None => db.query::<HistoryPage>().live().try_ids()?,
    };
    for id in &ids {
        db.delete::<HistoryPage>(*id)?;
    }
    db.save()?;
    Ok(ids.len())
}

// --- completion ----------------------------------------------------------------------------

/// The word completion narrows the library by: the typed text's first word, without a scheme
/// or `www.`, lowercased. Completion's own ranking does the rest.
pub fn narrowing_word(typed: &str) -> String {
    let lower = typed.trim().trim_start_matches('?').to_lowercase();
    let first = lower.split_whitespace().next().unwrap_or_default();
    first
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .to_string()
}

/// The history pages and bookmarks completion ranks for `typed`: everything whose address or
/// title holds the typed text's first word, the most visited first, each bookmark merged into
/// its history page.
pub fn candidates(
    db: &ModelContainer,
    typed: &str,
    limit: usize,
) -> Result<Vec<Candidate>, DbError> {
    let word = narrowing_word(typed);
    if word.is_empty() {
        return Ok(Vec::new());
    }
    let pages = db
        .query_raw::<HistoryPage>(
            format!(
                "SELECT id FROM history WHERE instr(day_fold(url), ?) > 0 OR instr(day_fold(title), ?) > 0 \
                 ORDER BY visits + 2 * typed DESC, last_visit DESC LIMIT {limit}"
            ),
            vec![Value::Text(word.clone()), Value::Text(word.clone())],
            &["history"],
        )
        .try_collect()?;
    let marks = db
        .query_raw::<Bookmark>(
            "SELECT id FROM bookmarks WHERE instr(day_fold(url), ?) > 0 OR instr(day_fold(title), ?) > 0 \
             ORDER BY position LIMIT 100",
            vec![Value::Text(word.clone()), Value::Text(word)],
            &["bookmarks"],
        )
        .try_collect()?;
    Ok(merge(pages, marks))
}

/// The recent history and every bookmark, for the start page's Frequently Visited.
pub fn recent_candidates(db: &ModelContainer, limit: usize) -> Result<Vec<Candidate>, DbError> {
    let pages = db
        .query::<HistoryPage>()
        .sort(HistoryPage::last_visit().desc())
        .limit(limit)
        .live()
        .try_collect()?;
    let marks = bookmarks(db)?;
    Ok(merge(pages, marks))
}

fn merge(pages: Vec<HistoryPage>, marks: Vec<Bookmark>) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = pages
        .into_iter()
        .map(|p| Candidate {
            url: p.url,
            title: p.title,
            visits: p.visits.clamp(0, i64::from(u32::MAX)) as u32,
            typed: p.typed.clamp(0, i64::from(u32::MAX)) as u32,
            last_visit: p.last_visit,
            bookmarked: false,
        })
        .collect();
    for b in marks {
        let key = address::history_key(&b.url).unwrap_or_else(|| b.url.clone());
        match out.iter_mut().find(|c| c.url == key) {
            Some(c) => {
                c.bookmarked = true;
                if !b.title.is_empty() {
                    c.title = b.title;
                }
            }
            None => out.push(Candidate {
                url: b.url,
                title: b.title,
                bookmarked: true,
                ..Default::default()
            }),
        }
    }
    out
}

// --- bookmarks -----------------------------------------------------------------------------

/// Every bookmark in order.
pub fn bookmarks(db: &ModelContainer) -> Result<Vec<Bookmark>, DbError> {
    db.query::<Bookmark>()
        .sort(Bookmark::position().asc())
        .live()
        .try_collect()
}

/// The bookmark for a page, matched by its history key so `#fragment`s and `www.` spelling of
/// the same address count as the same page.
pub fn bookmark_for(db: &ModelContainer, url: &str) -> Result<Option<Bookmark>, DbError> {
    let key = address::history_key(url).unwrap_or_else(|| url.to_string());
    Ok(bookmarks(db)?
        .into_iter()
        .find(|b| address::history_key(&b.url).unwrap_or_else(|| b.url.clone()) == key))
}

/// Bookmark a page at the end of the list. A page already bookmarked keeps its bookmark.
pub fn add_bookmark(db: &ModelContainer, url: &str, title: &str, now: i64) -> Result<u64, DbError> {
    if let Some(existing) = bookmark_for(db, url)? {
        return Ok(existing.id);
    }
    let all = bookmarks(db)?;
    let id = next_id::<Bookmark>(db, "bookmarks")?;
    db.insert(Bookmark {
        id,
        url: url.to_string(),
        title: title.trim().to_string(),
        position: all.last().map(|b| b.position + 1).unwrap_or(0),
        added: now,
    });
    db.save()?;
    Ok(id)
}

/// Rename a bookmark or change its address.
pub fn edit_bookmark(db: &ModelContainer, id: u64, title: &str, url: &str) -> Result<(), DbError> {
    if let Some(row) = db.try_get::<Bookmark>(id)? {
        row.title().write(title.trim().to_string());
        row.url().write(url.trim().to_string());
    }
    db.save()
}

pub fn remove_bookmark(db: &ModelContainer, id: u64) -> Result<(), DbError> {
    db.delete::<Bookmark>(id)?;
    db.save()
}

/// Move a bookmark to `index` in the list, renumbering the positions.
pub fn move_bookmark(db: &ModelContainer, id: u64, index: usize) -> Result<(), DbError> {
    let mut all = bookmarks(db)?;
    let Some(from) = all.iter().position(|b| b.id == id) else {
        return Ok(());
    };
    let b = all.remove(from);
    all.insert(index.min(all.len()), b);
    for (i, b) in all.iter().enumerate() {
        if b.position != i as i64
            && let Some(row) = db.try_get::<Bookmark>(b.id)?
        {
            row.position().write(i as i64);
        }
    }
    db.save()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000_000;

    fn memory() -> ModelContainer {
        open(Sqlite::memory()).expect("in-memory library")
    }

    #[test]
    fn visits_dedupe_count_and_keep_titles() {
        let db = memory();
        let a = record_visit(&db, "https://example.com/", Some("Example"), true, NOW).unwrap();
        let b = record_visit(&db, "https://example.com/", None, false, NOW + 5).unwrap();
        assert_eq!(a, b);
        let pages = history(&db, "", 10).unwrap();
        assert_eq!(pages.len(), 1);
        let p = &pages[0];
        assert_eq!(
            (p.visits, p.typed, p.last_visit, p.first_visit),
            (2, 1, NOW + 5, NOW)
        );
        assert_eq!(p.title, "Example");
        record_visit(&db, "https://example.com/", Some("Renamed"), false, NOW + 9).unwrap();
        assert_eq!(history(&db, "", 10).unwrap()[0].title, "Renamed");
        set_title(&db, "https://example.com/", "  ").unwrap();
        assert_eq!(history(&db, "", 10).unwrap()[0].title, "Renamed");
        set_title(&db, "https://example.com/", "Later").unwrap();
        assert_eq!(history(&db, "", 10).unwrap()[0].title, "Later");
    }

    #[test]
    fn history_is_newest_first_and_searchable() {
        let db = memory();
        record_visit(&db, "https://a.example/", Some("Alpha"), false, NOW).unwrap();
        record_visit(
            &db,
            "https://b.example/",
            Some("Bravo Café"),
            false,
            NOW + 1,
        )
        .unwrap();
        record_visit(&db, "https://c.example/", Some("Charlie"), false, NOW + 2).unwrap();
        let urls: Vec<_> = history(&db, "", 10)
            .unwrap()
            .into_iter()
            .map(|p| p.url)
            .collect();
        assert_eq!(
            urls,
            [
                "https://c.example/",
                "https://b.example/",
                "https://a.example/"
            ]
        );
        assert_eq!(
            history(&db, "CAFÉ", 10).unwrap()[0].url,
            "https://b.example/"
        );
        assert_eq!(history(&db, "a.example", 10).unwrap().len(), 1);
        assert_eq!(history(&db, "", 2).unwrap().len(), 2);
        assert!(history(&db, "zulu", 10).unwrap().is_empty());
    }

    #[test]
    fn clearing_by_range() {
        let db = memory();
        for (i, url) in [
            "https://old.example/",
            "https://mid.example/",
            "https://new.example/",
        ]
        .iter()
        .enumerate()
        {
            record_visit(&db, url, None, false, NOW + i as i64 * 1000).unwrap();
        }
        assert_eq!(clear_history(&db, Some(NOW + 1000)).unwrap(), 2);
        let left: Vec<_> = history(&db, "", 10)
            .unwrap()
            .into_iter()
            .map(|p| p.url)
            .collect();
        assert_eq!(left, ["https://old.example/"]);
        assert_eq!(clear_history(&db, None).unwrap(), 1);
        assert!(history(&db, "", 10).unwrap().is_empty());
        // Ids keep growing after a clear.
        assert!(record_visit(&db, "https://x.example/", None, false, NOW).unwrap() >= 1);
    }

    #[test]
    fn deleting_pages() {
        let db = memory();
        let a = record_visit(&db, "https://a.example/", None, false, NOW).unwrap();
        record_visit(&db, "https://b.example/", None, false, NOW).unwrap();
        delete_pages(&db, &[a]).unwrap();
        assert_eq!(history(&db, "", 10).unwrap()[0].url, "https://b.example/");
    }

    #[test]
    fn candidates_merge_bookmarks() {
        let db = memory();
        record_visit(&db, "https://github.com/", Some("GitHub"), true, NOW).unwrap();
        record_visit(&db, "https://gitlab.com/", Some("GitLab"), false, NOW).unwrap();
        record_visit(&db, "https://example.com/", Some("Example"), false, NOW).unwrap();
        add_bookmark(&db, "https://github.com/#top", "My GitHub", NOW).unwrap();
        add_bookmark(&db, "https://gitea.example/", "Gitea", NOW).unwrap();
        let c = candidates(&db, "https://www.GIT", 50).unwrap();
        assert_eq!(c.len(), 3);
        let gh = c.iter().find(|c| c.url == "https://github.com/").unwrap();
        assert!(gh.bookmarked);
        assert_eq!(gh.title, "My GitHub");
        assert_eq!(gh.typed, 1);
        let gitea = c
            .iter()
            .find(|c| c.url == "https://gitea.example/")
            .unwrap();
        assert!(gitea.bookmarked && gitea.visits == 0);
        // The most visited come first.
        assert_eq!(c[0].url, "https://github.com/");
        assert!(candidates(&db, "   ", 50).unwrap().is_empty());
        assert_eq!(narrowing_word("?Rust lang"), "rust");
    }

    #[test]
    fn recent_candidates_include_every_bookmark() {
        let db = memory();
        record_visit(&db, "https://a.example/", None, false, NOW).unwrap();
        add_bookmark(&db, "https://saved.example/", "Saved", NOW).unwrap();
        let c = recent_candidates(&db, 10).unwrap();
        assert_eq!(c.len(), 2);
        assert!(
            c.iter()
                .any(|c| c.url == "https://saved.example/" && c.bookmarked)
        );
    }

    #[test]
    fn bookmarks_order_edit_and_dedupe() {
        let db = memory();
        let a = add_bookmark(&db, "https://a.example/", " A ", NOW).unwrap();
        let b = add_bookmark(&db, "https://b.example/", "B", NOW).unwrap();
        let c = add_bookmark(&db, "https://c.example/", "C", NOW).unwrap();
        assert_eq!(
            add_bookmark(&db, "https://a.example/#x", "again", NOW).unwrap(),
            a
        );
        let titles = |db: &ModelContainer| {
            bookmarks(db)
                .unwrap()
                .into_iter()
                .map(|b| b.title)
                .collect::<Vec<_>>()
        };
        assert_eq!(titles(&db), ["A", "B", "C"]);
        move_bookmark(&db, c, 0).unwrap();
        assert_eq!(titles(&db), ["C", "A", "B"]);
        move_bookmark(&db, c, 99).unwrap();
        assert_eq!(titles(&db), ["A", "B", "C"]);
        edit_bookmark(&db, b, "Bee", "https://bee.example/").unwrap();
        assert_eq!(
            bookmark_for(&db, "https://bee.example/")
                .unwrap()
                .unwrap()
                .title,
            "Bee"
        );
        assert!(bookmark_for(&db, "https://b.example/").unwrap().is_none());
        remove_bookmark(&db, a).unwrap();
        assert_eq!(titles(&db), ["Bee", "C"]);
        move_bookmark(&db, 404, 0).unwrap();
    }
}
