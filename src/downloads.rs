// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0

//! Downloads: asking before a file is saved, saving it where the user looks for downloads, and
//! the list in the library that shows each file's progress and opens it.
//!
//! The web view offers a file it will not show (`WebView::on_download`); the list lives for the
//! whole app, since a download goes on after its tab closes. Finished downloads are remembered
//! across launches; one still running when the app quits is not.

use day::prelude::*;
use day_piece_webview::{DownloadEvent, DownloadRequest, DownloadResponder};
use netskip_model::tabs::format_bytes;
use serde::{Deserialize, Serialize};

use crate::browser::Browser;
use crate::library::Pane;
use crate::res;

const DOWNLOADS_KEY: &str = "netskip.downloads";
/// How many finished downloads the list remembers.
const REMEMBERED: usize = 200;

/// How one download stands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Running,
    Done,
    /// It stopped, with the engine's reason. `canceled` is the user's own doing.
    Failed(String),
}

/// One download in the list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Download {
    /// The web view's id for it while it runs, and the list's key afterwards.
    pub id: u64,
    pub name: String,
    pub url: String,
    /// Where the finished file is: what opens and removes it.
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub received: u64,
    #[serde(default)]
    pub total: Option<u64>,
    pub state: State,
}

impl Download {
    /// The line under the name: how far along, how large, or why it stopped.
    fn status(&self) -> String {
        match &self.state {
            State::Running => match self.total {
                Some(total) if total > 0 => {
                    res::str::download_progress(res::str::DownloadProgress {
                        received: format_bytes(self.received),
                        total: format_bytes(total),
                    })
                    .format()
                }
                _ => format_bytes(self.received),
            },
            State::Done => format_bytes(self.total.unwrap_or(self.received)),
            State::Failed(reason) if reason == "canceled" => res::str::download_canceled().format(),
            State::Failed(_) => res::str::download_failed().format(),
        }
    }
}

thread_local! {
    // `global`: created on first use, wherever that is, and never disposed with that scope.
    static LIST: Signal<Vec<Download>> = Signal::global(restore());
}

fn list_signal() -> Signal<Vec<Download>> {
    LIST.with(|l| *l)
}

/// The finished downloads of earlier launches, newest first. Ids restart from the top so they
/// cannot meet a running download's.
fn restore() -> Vec<Download> {
    if crate::fixtures::active() {
        return Vec::new();
    }
    let mut saved: Vec<Download> = day::prefs::get(DOWNLOADS_KEY)
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    for (index, item) in saved.iter_mut().enumerate() {
        item.id = u64::MAX - index as u64;
    }
    saved
}

fn save(list: &[Download]) {
    if crate::fixtures::active() {
        return;
    }
    let finished: Vec<&Download> = list
        .iter()
        .filter(|d| d.state == State::Done)
        .take(REMEMBERED)
        .collect();
    if let Ok(text) = serde_json::to_string(&finished) {
        day::prefs::set(DOWNLOADS_KEY, &text);
    }
}

fn update(id: u64, change: impl FnOnce(&mut Download)) {
    let list = list_signal();
    list.update(|all| {
        if let Some(item) = all.iter_mut().find(|d| d.id == id) {
            change(item);
        }
    });
}

/// A page offers a file: ask, as Safari does, then save it to the user's Downloads and show
/// the list, where its progress is.
pub fn offer(browser: Browser, request: DownloadRequest, responder: DownloadResponder) {
    day::task(async move {
        let size = request.size.map(format_bytes);
        let mut ask = confirm(res::str::download_confirm(request.suggested_name.clone()))
            .confirm_label(res::str::download());
        if let Some(size) = size {
            ask = ask.message(size);
        }
        if !ask.await {
            responder.cancel();
            return;
        }
        let id = request.id;
        list_signal().update(|all| {
            all.insert(
                0,
                Download {
                    id,
                    name: request.suggested_name.clone(),
                    url: request.url.clone(),
                    location: String::new(),
                    received: 0,
                    total: request.size,
                    state: State::Running,
                },
            );
        });
        let on_event = move |event: &DownloadEvent| match event {
            DownloadEvent::Progress { received, total } => update(id, |d| {
                d.received = *received;
                if total.is_some() {
                    d.total = *total;
                }
            }),
            DownloadEvent::Finished { location } => {
                update(id, |d| {
                    d.state = State::Done;
                    d.location = location.clone();
                    // The name it was saved under, where the system chose a free one.
                    if let Some(saved) = std::path::Path::new(location)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .filter(|_| !location.starts_with("content:"))
                    {
                        d.name = saved.to_string();
                    }
                    if d.received > 0 || d.total.is_none() {
                        d.total = Some(d.received);
                    }
                });
                list_signal().with_untracked(|all| save(all));
            }
            DownloadEvent::Failed(reason) => {
                update(id, |d| d.state = State::Failed(reason.clone()))
            }
        };
        // A fixture run saves into a folder of its own, emptied at every launch, so a script
        // finds the same file names each time and leaves nothing in the user's Downloads.
        match crate::fixtures::downloads_dir() {
            Some(dir) => responder.save_to(
                day_piece_webview::unused_path(&dir, &request.suggested_name),
                on_event,
            ),
            None => responder.save_to_downloads(&request.suggested_name, on_event),
        }
        browser.library.set(Some(Pane::Downloads));
    });
}

/// Stop a running download, or take a finished or failed one off the list. A finished file
/// stays where it was saved.
fn dismiss(item: &Download) {
    if item.state == State::Running {
        day_piece_webview::cancel_download(item.id);
        return;
    }
    let id = item.id;
    list_signal().update(|all| all.retain(|d| d.id != id));
    list_signal().with_untracked(|all| save(all));
}

/// Take every finished and failed download off the list. The files stay.
pub fn clear() {
    list_signal().update(|all| all.retain(|d| d.state == State::Running));
    list_signal().with_untracked(|all| save(all));
}

/// Whether anything is listed, for the Clear button.
pub fn any_finished() -> bool {
    list_signal().with(|all| all.iter().any(|d| d.state != State::Running))
}

/// The library's Downloads list: each file's name and how it stands, a button that stops a
/// running download or removes a row, and a tap that opens a finished file.
pub fn downloads_list(query: Signal<String>) -> impl Piece {
    let all = list_signal();
    // The library's search narrows the list by file name.
    let shown = move || {
        let q = query.get().trim().to_lowercase();
        let mut list = all.get();
        if !q.is_empty() {
            list.retain(|d| d.name.to_lowercase().contains(&q));
        }
        list
    };
    zstack((
        list(
            items(shown, |d: &Download| d.id),
            move |slot: ItemSlot<Download, u64>| {
                row((
                    column((
                        label(move || slot.field(|d| d.name.clone()))
                            .single_line()
                            .id_of(move || day::id!(download_name, slot.field(|d| d.name.clone()))),
                        label(move || slot.field(Download::status))
                            .font(Font::Caption)
                            .secondary()
                            .single_line()
                            .id_of(move || {
                                day::id!(download_status, slot.field(|d| d.name.clone()))
                            }),
                        when(
                            move || {
                                slot.field(|d| {
                                    d.state == State::Running && d.total.is_some_and(|t| t > 0)
                                })
                            },
                            move || {
                                progress(move || {
                                    slot.field(|d| match d.total {
                                        Some(total) if total > 0 => {
                                            (d.received as f64 / total as f64).min(1.0)
                                        }
                                        _ => 0.0,
                                    })
                                })
                                .grow(Dimension::Width)
                            },
                        ),
                    ))
                    .align(HAlign::Leading)
                    .spacing(2.0)
                    .grow(Dimension::Width),
                    button(move || {
                        if slot.field(|d| d.state == State::Running) {
                            res::str::cancel().format()
                        } else {
                            res::str::remove_download().format()
                        }
                    })
                    .icon(Symbol::Close)
                    .icon_only()
                    .plain()
                    .action(move || dismiss(&slot.get()))
                    .id_of(move || day::id!(download_dismiss, slot.field(|d| d.name.clone()))),
                ))
                .spacing(Space::S)
                .align(VAlign::Center)
                .padding(Insets::new(6.0, 4.0, 6.0, 4.0))
                .id_of(move || day::id!(download_row, slot.field(|d| d.name.clone())))
            },
        )
        .on_activate(move |id: u64| {
            let item = all.with_untracked(|a| a.iter().find(|d| d.id == id).cloned());
            if let Some(item) = item.filter(|d| d.state == State::Done) {
                day_piece_webview::open_download(&item.location);
            }
        })
        .inset_grouped()
        .id(day::id!(downloads_list)),
        when(
            move || shown().is_empty(),
            move || {
                label(move || {
                    if query.get().trim().is_empty() {
                        res::str::downloads_empty().format()
                    } else {
                        res::str::no_matches().format()
                    }
                })
                .secondary()
                .aligned(Alignment::Center)
                .id(day::id!(downloads_empty))
            },
        ),
    ))
    .grow(Dimension::Both)
}
