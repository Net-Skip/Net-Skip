// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
import SkipWeb
import NetSkipModel

#if SKIP || os(iOS)

@MainActor struct BrowserView: View {
    let configuration: WebEngineConfiguration
    let store: WebBrowserStore
    let submitURL: (String) -> ()
    @Binding var viewModel: BrowserViewModel

    @Binding var showSettings: Bool
    @Binding var showBottomBar: Bool

    @State var currentSuggestions: SearchSuggestions?
    @State var triggerPageLoadHaptic: Bool = false
    @State var pendingDownload: WebDownloadRequest? = nil
    @State var showDownloadPrompt: Bool = false
    #if SKIP
    @State var urlSelection: TextSelection? = nil
    #endif

    @Environment(NetSkipSettings.self) var settings

    @FocusState var isURLBarFocused: Bool

    var state: WebViewState {
        self.viewModel.state
    }

    /// Returns the current browser's webview
    var webView: PlatformWebView? {
        self.viewModel.navigator.webEngine?.webView
    }

    /// The display name shown in the download confirmation dialog — runs
    /// the same `resolvedFilename(for:)` correction the manager will use
    /// so the prompt matches the actual saved filename (e.g. Android's
    /// `appindex.bin` → `appindex.json`).
    var pendingDownloadDisplayName: String {
        if let request = pendingDownload {
            return NetSkipDownloadManager.resolvedFilename(for: request)
        }
        return "file"
    }

    var body: some View {
        // Total height of the bottom chrome (URL bar + toolbar) — the
        // WebView is padded by this amount so its scroll content stops
        // exactly where the chrome begins. On scroll-down the chrome
        // collapses to zero, the padding follows, and the WebView
        // takes over the full screen.
        let chromeHeight = showBottomBar
            ? (BrowserTabView.urlBarHeight + BrowserTabView.bottomToolbarHeight)
            : 0.0

        ZStack(alignment: .bottom) {
            ZStack {
                WebView(configuration: configuration, navigator: viewModel.navigator, state: $viewModel.state, onNavigationCommitted: {
                    logger.log("onNavigationCommitted")
                }, onDownloadRequested: { request in
                    logger.log("download requested: \(String(describing: request.url))")
                    Task { @MainActor in
                        if settings.promptForDownloads {
                            self.pendingDownload = request
                            self.showDownloadPrompt = true
                        } else {
                            NetSkipDownloadManager.shared.enqueue(request)
                        }
                    }
                })
                let showSuggestions = state.pageURL == nil || isURLBarFocused
                if showSuggestions {
                    suggestionsView()
                        .frame(maxHeight: .infinity)
                }
            }
            // Reserve space at the bottom for the chrome (URL bar +
            // toolbar). The WebView's scroll area ends here, so page
            // content (including the very last line on long pages) is
            // never hidden behind the chrome. Animated transition makes
            // it slide smoothly when scrolling toggles the chrome.
            .padding(.bottom, chromeHeight)

            // URL bar overlay sitting flush against the toolbar. The
            // bottom-padding lifts it by `bottomToolbarHeight` so its
            // bottom edge touches the toolbar's top edge.
            urlBarView()
                .frame(height: showBottomBar ? BrowserTabView.urlBarHeight : 0.0)
                .opacity(showBottomBar ? 1.0 : 0.0)
                .clipped()
                .padding(.bottom, showBottomBar ? BrowserTabView.bottomToolbarHeight : 0.0)
        }
        .frame(maxHeight: .infinity)
        .confirmationDialog(
            Text("Download \(pendingDownloadDisplayName)?",
                 bundle: .module,
                 comment: "title for the file-download confirmation dialog; argument is the filename"),
            isPresented: $showDownloadPrompt,
            titleVisibility: .visible
        ) {
            Button {
                if let request = pendingDownload {
                    NetSkipDownloadManager.shared.enqueue(request)
                }
                pendingDownload = nil
            } label: {
                Text("Download", bundle: .module, comment: "confirm-download button on the download confirmation dialog")
            }
            .accessibilityIdentifier("button.download.confirm")
            Button(role: .cancel) {
                pendingDownload = nil
            } label: {
                Text("Cancel", bundle: .module, comment: "cancel button on the download confirmation dialog")
            }
            .accessibilityIdentifier("button.download.cancel")
        }
        #if !SKIP
        .sensoryFeedback(.impact, trigger: triggerPageLoadHaptic)
        #endif
        .onAppear {
            updateWebView()
        }
        .onChange(of: settings.enableJavaScript, initial: false, { _, _ in self.updateWebView() })
        .onChange(of: settings.requestDesktopSite, initial: false, { _, _ in self.updateWebView() })
        .onChange(of: settings.textZoom, initial: false, { _, _ in self.applyTextZoom() })
    }

    struct SearchSuggestions : Identifiable {
        var id: SearchEngine.ID { engine.id }
        let engine: SearchEngine
        let suggestions: [String]
    }

    private static let desktopUserAgent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15"

    /// Synchronize settings with the current platform web view
    private func updateWebView() {
        #if !SKIP
        guard let webView = self.webView else {
            logger.warning("updateWebView: no web view")
            return
        }

        if let url = webView.url {
            updatePageURL(nil, url.absoluteString)
        }

        webView.configuration.defaultWebpagePreferences.allowsContentJavaScript = settings.enableJavaScript

        // Apply user agent
        if settings.requestDesktopSite {
            webView.customUserAgent = Self.desktopUserAgent
        } else if !settings.userAgent.isEmpty {
            webView.customUserAgent = settings.userAgent
        } else {
            webView.customUserAgent = nil // use default
        }
        #endif
    }

    func updatePageURL(_ oldURL: String?, _ newURL: String?) {
        if let newURL = newURL {
            logger.log("changed pageURL to: \(newURL)")
            viewModel.urlTextField = newURL
            // Mirror onto the view-model's saved field so the tab card
            // in the overview still shows the right domain after the
            // user leaves this tab (the WebView's live `state.url` is
            // not always populated for background tabs).
            viewModel.savedURL = newURL
            showBottomBar = true // when the URL changes, always show the bottom bar again
            if var pageInfo = trying(operation: { try store.loadItems(type: .active, ids: [viewModel.id]) })?.first {
                pageInfo.url = newURL
                _ = trying { try store.saveItems(type: .active, items: [pageInfo]) }
            }
        }
    }

    func updatePageTitle(_ oldTitle: String?, _ newTitle: String?) {
        if let newTitle = newTitle {
            logger.log("loaded page title: \(newTitle)")
            // Same mirror as `updatePageURL` — keeps the tab card
            // showing the page title in the overview after the user
            // navigates away to another tab.
            viewModel.savedTitle = newTitle
            addPageToHistory()
            if var pageInfo = trying(operation: { try store.loadItems(type: .active, ids: [viewModel.id]) })?.first {
                pageInfo.title = newTitle
                _ = trying {
                    try store.saveItems(type: .active, items: [pageInfo])
                }
            }
            // Fire page load haptic
            if settings.pageLoadHaptics {
                triggerPageLoadHaptic.toggle()
            }
        }
    }

    private func applyTextZoom() {
        #if !SKIP
        // iOS: WKWebView.pageZoom on iOS scales the viewport rather than the content,
        // so values > 1.0 make content appear smaller. Use the reciprocal to get
        // the expected behavior where textZoom > 1.0 means "zoom in" (larger text).
        if let webView = self.webView {
            webView.pageZoom = 1.0 / settings.textZoom
        }
        #else
        // Android: use the native WebView settings.textZoom (integer percentage)
        if let webView = self.webView {
            let pct = Int(settings.textZoom * 100.0)
            webView.getSettings().setTextZoom(pct)
        }
        #endif
    }

    func urlBarView() -> some View {
        VStack(spacing: 0.0) {
            if showBottomBar {
                Divider()
            }
            urlBarComponentView()
        }
            #if !SKIP
            .onChange(of: state.pageURL, updatePageURL)
            .onChange(of: state.pageTitle, updatePageTitle)
            #else
            // workaround onChange() expects an Equatable, which Optional does not conform to
            // https://github.com/skiptools/skip-ui/issues/27
            .onChange(of: state.pageURL ?? fallbackURL) {
                updatePageURL($0, $1)
            }
            .onChange(of: state.pageTitle ?? "SENTINEL_TITLE") {
                updatePageTitle($0, $1)
            }
            #endif
    }

    /// Extracts just the domain from a URL string for display
    func domainFromURL(_ urlString: String) -> String {
        guard let url = URL(string: urlString) else { return urlString }
        var host = url.host ?? urlString
        if host.hasPrefix("www.") {
            host = host.replacingOccurrences(of: "www.", with: "")
        }
        return host
    }

    /// The URL bar TextField. The Skip variant binds `selection` so we can
    /// programmatically select-all-on-focus to mirror iOS UITextField behavior.
    @ViewBuilder func urlTextFieldView() -> some View {
        #if SKIP
        TextField(text: $viewModel.urlTextField, selection: $urlSelection) {
            Text(isURLBarFocused ? "Search or enter website name" : "")
        }
        #else
        TextField(text: $viewModel.urlTextField) {
            Text(isURLBarFocused ? "Search or enter website name" : "")
        }
        #endif
    }

    @ViewBuilder func urlBarComponentView() -> some View {
        ZStack(alignment: .center) {
            // Background capsule with progress bar
            ZStack(alignment: .bottom) {
                Capsule()
                    .fill(showBottomBar ? urlBarBackground : .clear)

                ProgressView(value: state.estimatedProgress ?? 0.0)
                    .progressViewStyle(.linear)
                    .frame(height: 2.0)
                    .tint(.accentColor)
                    .opacity(state.isLoading ? 1.0 : 0.0)
            }
            .frame(height: showBottomBar ? 44.0 : 25.0)
            .padding(.top, 4.0)
            .padding(.bottom, 0.0)

            // The TextField is ALWAYS in the view hierarchy so taps
            // always land on it and @FocusState works on iOS.
            // When not focused, we render its text as .clear so the
            // domain overlay is visible instead. We do NOT use
            // .opacity(0) because iOS skips hit testing for invisible views.
            HStack(spacing: 6) {
                urlTextFieldView()
                .textFieldStyle(.plain)
                .font(.system(size: isURLBarFocused ? 16.0 : 15.0))
                .foregroundStyle(isURLBarFocused ? Color.primary : Color.clear)
                .accessibilityIdentifier("field.url")
                #if !SKIP
                #if os(iOS)
                .keyboardType(.webSearch)
                .textContentType(.URL)
                .autocorrectionDisabled(true)
                .textInputAutocapitalization(.never)
                .multilineTextAlignment(isURLBarFocused ? .leading : .center)
                // Select-all on focus so a tap lets the user immediately
                // type a replacement URL (mirrors the Android branch's
                // `TextSelection` behavior below). We filter by the
                // SwiftUI accessibility identifier rather than the
                // @FocusState — at the moment `textDidBeginEditing` fires
                // the UIKit first-responder change hasn't yet flushed
                // into @FocusState, so the previous `guard isURLBarFocused`
                // dropped the very first tap and the user saw the cursor
                // placed mid-text instead of a full selection.
                .onReceive(NotificationCenter.default.publisher(for: UITextField.textDidBeginEditingNotification)) { obj in
                    guard let textField = obj.object as? UITextField,
                          textField.accessibilityIdentifier == "field.url",
                          !(textField.text ?? "").isEmpty else { return }
                    textField.selectAll(nil)
                }
                #endif
                #endif
                .focused($isURLBarFocused)
                .onChange(of: isURLBarFocused) { _, newValue in
                    if newValue {
                        showBottomBar = true
                        #if SKIP
                        // Pre-select all text in URL bar when focused (mirrors iOS UITextField.selectAll behavior)
                        let text = viewModel.urlTextField
                        if !text.isEmpty {
                            urlSelection = TextSelection(range: text.startIndex..<text.endIndex)
                        }
                        #endif
                    }
                }
                .onSubmit {
                    self.submitURL(viewModel.urlTextField)
                    self.isURLBarFocused = false
                }
                .task(id: viewModel.urlTextField) {
                    if settings.searchSuggestions && isURLBarFocused && !viewModel.urlTextField.isEmpty {
                        Task {
                            do {
                                try await fetchSearchSuggestions(string: viewModel.urlTextField)
                            } catch {
                                logger.error("error issuing search suggestion query: \(error)")
                            }
                        }
                    }
                }

                if isURLBarFocused {
                    Button(action: { self.viewModel.urlTextField = "" }, label: {
                        Image("xmark.circle.fill", bundle: .module)
                            .foregroundStyle(.secondary)
                            #if !SKIP
                            .symbolRenderingMode(.hierarchical)
                            #endif
                    })
                    .buttonStyle(.plain)
                    .accessibilityIdentifier("button.url.clear")
                    .accessibilityLabel(Text("Clear URL", bundle: .module, comment: "accessibility label for the URL bar clear button"))
                } else if state.isLoading {
                    // Stop loading button
                    Button(action: { self.viewModel.navigator.stopLoading() }, label: {
                        Image("xmark", bundle: .module)
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                    })
                    .buttonStyle(.plain)
                    .accessibilityIdentifier("button.url.stop")
                    .accessibilityLabel(Text("Stop loading", bundle: .module, comment: "accessibility label for the URL bar stop-loading button"))
                } else if state.pageURL != nil {
                    // Reload button — tap reloads normally, long-press
                    // reveals a "Hard Reload" item that clears the cache
                    // before reloading. Mirrors the long-press menus on
                    // the Tabs / back / forward toolbar buttons.
                    Menu {
                        Button(action: { hardReloadAction() }) {
                            Label {
                                Text("Hard Reload", bundle: .module, comment: "menu label that clears the cache for the current page and reloads it")
                            } icon: {
                                Image("arrow.clockwise", bundle: .module)
                            }
                        }
                        .accessibilityIdentifier("menu.hardReload")
                    } label: {
                        Image("arrow.clockwise", bundle: .module)
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                    } primaryAction: {
                        self.viewModel.navigator.reload()
                    }
                    .accessibilityIdentifier("button.url.reload")
                    .accessibilityLabel(Text("Reload page", bundle: .module, comment: "accessibility label for the URL bar reload button"))
                }
            }
            .padding(.horizontal, 12.0)

            // Domain overlay: visible only when NOT focused.
            // allowsHitTesting(false) lets taps fall through to the TextField.
            if !isURLBarFocused {
                HStack(spacing: 4) {
                    if state.pageURL != nil && !state.isLoading {
                        Image("lock", bundle: .module)
                            .font(.system(size: 12))
                            .foregroundStyle(.secondary)
                    }

                    if state.isLoading {
                        Text("Loading...")
                            .font(.system(size: 15))
                            .foregroundStyle(.secondary)
                    } else if let pageURL = state.pageURL {
                        Text(domainFromURL(pageURL))
                            .font(.system(size: 15))
                            .foregroundStyle(.primary)
                            .lineLimit(1)
                    } else {
                        Text("Search or enter website name", bundle: .module, comment: "placeholder string for URL bar")
                            .font(.system(size: 15))
                            .foregroundStyle(.secondary)
                    }
                }
                .allowsHitTesting(false)
                .frame(width: showBottomBar ? nil : 0.0)
                .opacity(showBottomBar ? 1.0 : 0.0)
            }
        }
        .padding(.horizontal, showBottomBar ? 8.0 : 0.0)
    }

    @ViewBuilder func suggestionsView() -> some View {
        VStack(spacing: 0) {
            // Top bar with Cancel — only show when the URL bar is actually focused,
            // so the button has a meaningful effect. On a new tab with no page loaded
            // (state.pageURL == nil), the suggestionsView is visible by default; showing
            // Cancel there would be non-functional because there's nothing to cancel.
            if isURLBarFocused {
                HStack {
                    Spacer()
                    Button(action: {
                        // Defocus the URL bar (dismisses keyboard) and clear any in-flight
                        // search suggestions so the dropdown collapses immediately. Also
                        // restore the URL bar text to the current page URL (or clear it
                        // for a new tab) so the user's in-progress edit is discarded.
                        self.isURLBarFocused = false
                        self.currentSuggestions = nil
                        self.viewModel.urlTextField = state.pageURL ?? ""
                    }, label: {
                        Text("Cancel", bundle: .module, comment: "cancel button for dismissing suggestions")
                            .fontWeight(.medium)
                    })
                    .buttonStyle(.plain)
                    .accessibilityIdentifier("button.url.cancel")
                    .padding(.trailing, 16)
                    .padding(.top, 12)
                }
            }

            // Filter out any suggestion that exactly matches what's already in the URL bar.
            // The search engine often echoes the typed URL back as a suggestion, producing a
            // visual "duplicate URL bar" effect (the suggestion row at the top looks identical
            // to the URL bar at the bottom).
            let typed = viewModel.urlTextField
            let filteredSuggestions = (currentSuggestions?.suggestions ?? []).filter { $0 != typed }
            if !filteredSuggestions.isEmpty {
                List {
                    ForEach(Array(filteredSuggestions.enumerated()), id: \.0) { (index, suggestion) in
                        Button {
                            withAnimation {
                                self.submitURL(suggestion)
                                self.isURLBarFocused = false
                            }
                        } label: {
                            HStack(spacing: 10) {
                                Image("magnifyingglass", bundle: .module)
                                    .foregroundStyle(.secondary)
                                    .font(.system(size: 14))
                                Text(suggestion)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                            #if !SKIP
                            .contentShape(Rectangle())
                            #endif
                        }
                        .buttonStyle(.plain)
                    }
                }
                .listStyle(.plain)
            } else {
                Spacer()
                VStack(spacing: 12) {
                    Image("magnifyingglass", bundle: .module)
                        .font(.system(size: 36))
                        .foregroundStyle(.secondary)
                    Text("Search or enter a URL", bundle: .module, comment: "start page prompt")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                }
                Spacer()
            }
        }
        #if !SKIP
        .background(Color(UIColor.systemBackground))
        #endif
    }

    func fetchSearchSuggestions(string: String) async throws {
        guard let engine = SearchEngine.lookup(id: settings.searchEngine) else { return }
        // SKIP NOWARN
        let suggestions: [String]? = try await engine.suggestions(string)
        logger.log("fetched search suggestion: \(String(describing: suggestions))")
        self.currentSuggestions = SearchSuggestions(engine: engine, suggestions: suggestions ?? [])
    }

    /// Clears the current tab's cache (disk + memory + offline application
    /// cache, but not cookies or local storage) and reloads the page. Same
    /// data-type set as the Settings "Clear Web Cache" action, so the user
    /// stays signed in everywhere; this just forces fresh asset fetches for
    /// the page in front of them, the way a desktop "hard reload" does.
    func hardReloadAction() {
        logger.info("hardReloadAction")
        if let engine = self.viewModel.navigator.webEngine {
            let cacheTypes: Set<WebSiteDataType> = [.diskCache, .memoryCache, .offlineWebApplicationCache]
            let navigator = self.viewModel.navigator
            Task {
                do {
                    try await engine.removeData(ofTypes: cacheTypes, modifiedSince: .distantPast)
                } catch {
                    logger.warning("hardReloadAction: failed to clear cache: \(error)")
                }
                navigator.reload()
            }
        } else {
            self.viewModel.navigator.reload()
        }
    }

    func addPageToHistory() {
        if let url = state.pageURL, let title = state.pageTitle {
            logger.info("addPageToHistory: \(title) \(url)")
            trying {
                // Update existing history entry if URL already exists, otherwise create new
                if let netStore = store as? NetSkipWebBrowserStore {
                    if try netStore.updateExistingItem(type: .history, url: url, title: title) != nil {
                        return // updated existing entry
                    }
                }
                _ = try store.saveItems(type: .history, items: [PageInfo(url: url, title: title)])
            }
        }
    }
}
#endif
