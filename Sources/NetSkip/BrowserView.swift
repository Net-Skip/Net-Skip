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

    @Binding var searchEngine: SearchEngine.ID
    @Binding var searchSuggestions: Bool
    @Binding var showSettings: Bool
    @Binding var showBottomBar: Bool
    @Binding var userAgent: String
    @Binding var enableJavaScript: Bool
    @Binding var pageLoadHaptics: Bool
    @Binding var requestDesktopSite: Bool
    @Binding var textZoom: Double

    @State var currentSuggestions: SearchSuggestions?
    @State var triggerPageLoadHaptic: Bool = false

    @FocusState var isURLBarFocused: Bool

    var state: WebViewState {
        self.viewModel.state
    }

    /// Returns the current browser's webview
    var webView: PlatformWebView? {
        self.viewModel.navigator.webEngine?.webView
    }

    var body: some View {
        VStack(spacing: 0.0) {
            urlBarView()
            ZStack {
                WebView(configuration: configuration, navigator: viewModel.navigator, state: $viewModel.state, onNavigationCommitted: {
                    logger.log("onNavigationCommitted")
                })
                let showSuggestions = state.pageURL == nil || isURLBarFocused
                if showSuggestions {
                    suggestionsView()
                        .frame(maxHeight: .infinity)
                }
            }
        }
        .frame(maxHeight: .infinity)
        #if !SKIP
        .sensoryFeedback(.impact, trigger: triggerPageLoadHaptic)
        #endif
        .onAppear {
            updateWebView()
        }
        .onChange(of: enableJavaScript, initial: false, { _, _ in self.updateWebView() })
        .onChange(of: requestDesktopSite, initial: false, { _, _ in self.updateWebView() })
        .onChange(of: textZoom, initial: false, { _, _ in self.applyTextZoom() })
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

        webView.configuration.defaultWebpagePreferences.allowsContentJavaScript = enableJavaScript

        // Apply user agent
        if requestDesktopSite {
            webView.customUserAgent = Self.desktopUserAgent
        } else if !userAgent.isEmpty {
            webView.customUserAgent = userAgent
        } else {
            webView.customUserAgent = nil // use default
        }
        #endif
    }

    func updatePageURL(_ oldURL: String?, _ newURL: String?) {
        if let newURL = newURL {
            logger.log("changed pageURL to: \(newURL)")
            viewModel.urlTextField = newURL
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
            addPageToHistory()
            if var pageInfo = trying(operation: { try store.loadItems(type: .active, ids: [viewModel.id]) })?.first {
                pageInfo.title = newTitle
                _ = trying {
                    try store.saveItems(type: .active, items: [pageInfo])
                }
            }
            // Fire page load haptic
            if pageLoadHaptics {
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
            webView.pageZoom = 1.0 / textZoom
        }
        #else
        // Android: use the native WebView settings.textZoom (integer percentage)
        if let webView = self.webView {
            let pct = Int(textZoom * 100.0)
            webView.getSettings().setTextZoom(pct)
        }
        #endif
    }

    func urlBarView() -> some View {
        VStack(spacing: 0.0) {
            urlBarComponentView()
            if showBottomBar {
                Divider()
            }
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
                TextField(text: $viewModel.urlTextField) {
                    Text(isURLBarFocused ? "Search or enter website name" : "")
                }
                .textFieldStyle(.plain)
                .font(.system(size: isURLBarFocused ? 16.0 : 15.0))
                .foregroundStyle(isURLBarFocused ? Color.primary : Color.clear)
                #if !SKIP
                #if os(iOS)
                .keyboardType(.webSearch)
                .textContentType(.URL)
                .autocorrectionDisabled(true)
                .textInputAutocapitalization(.never)
                .multilineTextAlignment(isURLBarFocused ? .leading : .center)
                .onReceive(NotificationCenter.default.publisher(for: UITextField.textDidBeginEditingNotification)) { obj in
                    if let textField = obj.object as? UITextField {
                        textField.selectAll(nil)
                    }
                }
                #endif
                #endif
                .focused($isURLBarFocused)
                .onChange(of: isURLBarFocused) { _, newValue in
                    if newValue {
                        showBottomBar = true
                    }
                }
                .onSubmit {
                    self.submitURL(viewModel.urlTextField)
                    self.isURLBarFocused = false
                }
                .task(id: viewModel.urlTextField) {
                    if searchSuggestions && isURLBarFocused && !viewModel.urlTextField.isEmpty {
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
                } else if state.isLoading {
                    // Stop loading button
                    Button(action: { self.viewModel.navigator.stopLoading() }, label: {
                        Image("xmark", bundle: .module)
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                    })
                    .buttonStyle(.plain)
                } else if state.pageURL != nil {
                    // Reload button
                    Button(action: { self.viewModel.navigator.reload() }, label: {
                        Image("arrow.clockwise", bundle: .module)
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                    })
                    .buttonStyle(.plain)
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
            // Top bar with cancel
            HStack {
                Spacer()
                Button(action: {
                    withAnimation {
                        self.isURLBarFocused = false
                    }
                }, label: {
                    Text("Cancel", bundle: .module, comment: "cancel button for dismissing suggestions")
                        .fontWeight(.medium)
                })
                .buttonStyle(.plain)
                .padding(.trailing, 16)
                .padding(.top, 12)
            }

            if let suggestions = currentSuggestions, !suggestions.suggestions.isEmpty {
                List {
                    ForEach(Array(suggestions.suggestions.enumerated()), id: \.0) { (index, suggestion) in
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
        guard let engine = SearchEngine.lookup(id: self.searchEngine) else { return }
        // SKIP NOWARN
        let suggestions: [String]? = try await engine.suggestions(string)
        logger.log("fetched search suggestion: \(String(describing: suggestions))")
        self.currentSuggestions = SearchSuggestions(engine: engine, suggestions: suggestions ?? [])
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
