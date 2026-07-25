// SPDX-License-Identifier: GPL-2.0-or-later
//
// `BrowserViewModel` — the per-tab observable model. Owns the
// navigator, the live WebView state, and the saved* mirrors that
// keep tab cards populated when SwiftUI tears down a backgrounded
// tab's WKWebView and the live state momentarily reads as
// `about:blank`.

import SwiftUI
import SkipWeb
import NetSkipModel

#if SKIP || os(iOS)

@Observable public class BrowserViewModel: Identifiable {
    /// The persistent ID of the page.
    // SKIP INSERT: @Suppress("MUST_BE_INITIALIZED_OR_FINAL_OR_ABSTRACT") // var to workaround Kotlin to 2 error: "Property must be initialized, be final, or be abstract."
    public let id: PageInfo.ID
    let navigator: WebViewNavigator
    let configuration: WebEngineConfiguration
    let store: WebBrowserStore
    var state = WebViewState()
    var urlTextField: String = ""

    /// Saved metadata from the database, used as fallback before the page loads.
    var savedTitle: String = ""
    var savedURL: String = ""

    /// User-pinned tab — shows a pin badge on its tab card and stays
    /// behind a "Closing a pinned tab" confirmation.
    var isPinned: Bool = false

    /// One-shot signal set by `newTabAction` when the user creates (or
    /// reuses) a blank tab — `BrowserView` honors it by focusing the
    /// URL bar so the keyboard comes up and the user can type their
    /// query immediately, then clears the flag.
    var shouldFocusURLBar: Bool = false

    /// True when the WebView is currently rendering the
    /// Readability-extracted article view instead of the live page.
    /// Cleared automatically when the tab navigates to a different
    /// URL (see `BrowserView.updatePageURL`) so reader mode never
    /// leaks across navigations.
    var inReaderMode: Bool = false

    /// True for tabs created via "New Private Tab". A private tab is
    /// backed by a `WebProfile.ephemeral` configuration (so cookies /
    /// local storage live in an in-memory `WKWebsiteDataStore` /
    /// Android profile), its page loads are NOT recorded in the
    /// history table, and it is NOT persisted across launches.
    /// Immutable for the tab's lifetime — flipping the flag would
    /// leave the wrong underlying data store attached to the WebView.
    public let isPrivate: Bool

    public init(id: PageInfo.ID, navigator: WebViewNavigator, configuration: WebEngineConfiguration, store: WebBrowserStore, isPrivate: Bool = false) {
        self.id = id
        self.navigator = navigator
        self.configuration = configuration
        self.store = store
        self.isPrivate = isPrivate
    }

    /// PNG path on disk where this tab's grid-overview snapshot lives.
    /// Stored under the Caches directory so iOS / Android may evict it
    /// under storage pressure without breaking the app.
    public static func snapshotPath(for tabId: PageInfo.ID) -> URL {
        let dir = URL.cachesDirectory.appendingPathComponent("tab-snapshots")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("\(tabId).png")
    }

    /// Capture a thumbnail of the currently-attached `WKWebView` /
    /// Android `WebView` and write it to `snapshotPath(for: id)`.
    /// Called on page-load completion so the snapshot is fresh when
    /// the user opens the tab grid — capturing on tab switch is too
    /// late on iOS, where the leaving tab's WKWebView is already
    /// detached and `takeSnapshot` returns blank pixels.
    @MainActor
    public func captureSnapshot() async {
        guard let webEngine = navigator.webEngine else { return }
        do {
            let config = SkipWebSnapshotConfiguration(snapshotWidth: 300, imageFormat: .png)
            let snapshot = try await webEngine.takeSnapshot(configuration: config)
            try snapshot.imageData.write(to: Self.snapshotPath(for: id))
        } catch {
            // Best-effort — a missing snapshot just falls back to the
            // domain-letter avatar in the tab grid.
        }
    }
}

struct MiniAppLaunchItem: Identifiable {
    let appID: String
    var id: String { appID }
}

#endif
