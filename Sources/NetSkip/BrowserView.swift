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
    @Binding var blockAds: Bool
    @Binding var enableJavaScript: Bool

    /// Whether content rules are currently enabled
    @State var contentRulesEnabled: Bool = false
    @State var currentSuggestions: SearchSuggestions?

    #if !SKIP
    @FocusState var isURLBarFocused: Bool
    #else
    @State var isURLBarFocused: Bool = false
    #endif

    var state: WebViewState {
        self.viewModel.state
    }

    /// Returns the current browser's webview
    var webView: PlatformWebView? {
        self.viewModel.navigator.webEngine?.webView
    }

    var body: some View {
        VStack(spacing: 0.0) {
            ZStack {
                WebView(configuration: configuration, navigator: viewModel.navigator, state: $viewModel.state, onNavigationCommitted: {
                    logger.log("onNavigationCommitted")
                })
                suggestionsView()
                    .frame(maxHeight: .infinity)
                    .opacity(state.pageURL == nil || isURLBarFocused ? 1.0 : 0.0)
                    .animation(.easeIn, value: isURLBarFocused)
            }
            urlBarView()
        }
        .frame(maxHeight: .infinity)
        .onAppear {
            // synchronize web view with preferences
            updateWebView()
        }
        .onChange(of: enableJavaScript, initial: false, { _, _ in self.updateWebView() })
        .onChange(of: blockAds, initial: false, { _, _ in self.updateWebView() })
        #if !SKIP
        .onReceive(NotificationCenter.default.publisher(for: .webContentRulesLoaded).receive(on: DispatchQueue.main)) { _ in
            logger.log("reveived webContentRulesLoaded")
            // wen content rules are loaded asynchonously, so refresh them when they are loaded
            updateWebView()
        }
        #endif
    }

    struct SearchSuggestions : Identifiable {
        var id: SearchEngine.ID { engine.id }
        let engine: SearchEngine
        let suggestions: [String]
    }

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

        if blockAds == true && self.contentRulesEnabled == false {
            WebContentRuleListStore.default().getAvailableContentRuleListIdentifiers { contentRuleIDs in
                guard let contentRuleID = contentRuleIDs?.first else {
                    logger.log("updateWebView: content rules: no identifiers found")
                    return
                }
                logger.log("updateWebView: content rules: enabling \(contentRuleID)")
                WebContentRuleListStore.default().lookUpContentRuleList(forIdentifier: contentRuleID) { contentRuleList, error in
                    logger.log("updateWebView: lookup content rule \(contentRuleID) \(contentRuleList) \(error)")
                    if let contentRuleList = contentRuleList {
                        DispatchQueue.main.async {
                            logger.log("updateWebView: activating contentRuleList: \(contentRuleList)")
                            webView.configuration.userContentController.add(contentRuleList)
                            self.contentRulesEnabled = true
                        }
                    }
                }
            }
        } else if blockAds == false && self.contentRulesEnabled == true {
            logger.log("updateWebView: content rules: disabling content rules")
            self.contentRulesEnabled = false
            webView.configuration.userContentController.removeAllContentRuleLists()
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
        }
    }

    func urlBarView() -> some View {
        urlBarComponentView()
            //.background(urlBarBackground)
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

    @ViewBuilder func urlBarComponentView() -> some View {
        ZStack(alignment: .center) {
            ZStack(alignment: .bottom) {
                RoundedRectangle(cornerRadius: 5.0, style: .continuous)
                    .fill(showBottomBar ? urlBarBackground : .clear)

                ProgressView(value: state.estimatedProgress ?? 0.0)
                    .progressViewStyle(.linear)
                    .frame(height: 0.2) // thin progress bar
                    .opacity(state.isLoading ? 1.0 : 0.0)
            }
            .frame(height: showBottomBar ? 40.0 : 25.0)
            //.shadow(radius: 3.0) // grays out the URL bar while scrolling for some reason
            .padding(.vertical, 4.0)

            HStack {
                if !isURLBarFocused {
                    Button(action: { self.viewModel.navigator.reload() }, label: {
                        Image("textformat.size", bundle: .module)
                    })
                    .buttonStyle(.plain)
                    .frame(width: showBottomBar ? nil : 0.0) // hide button when the bottom bar is hidden
                    .opacity(showBottomBar ? 1.0 : 0.0) // hide button when the bottom bar is hidden
                }

                TextField(text: $viewModel.urlTextField) {
                    Text("Search or enter website name", bundle: .module, comment: "placeholder string for URL bar")
                }
                .textFieldStyle(.plain)
                //.font(Font.body)
                #if !SKIP
                #if os(iOS)
                .animation(.none, value: showBottomBar)
                .textScale(.secondary, isEnabled: !showBottomBar)
                .multilineTextAlignment(isURLBarFocused ? .leading : .center)
                .truncationMode(.middle)
                .keyboardType(.webSearch)
                .textContentType(.URL)
                .autocorrectionDisabled(true)
                .textInputAutocapitalization(.never)
                //.toolbar {
                //    ToolbarItemGroup(placement: .keyboard) {
                //        Button("XXX") {
                //            logger.log("Clicked Custom Search…")
                //        }
                //    }
                //}
                .onReceive(NotificationCenter.default.publisher(for: UITextField.textDidBeginEditingNotification)) { obj in
                    logger.log("received textDidBeginEditingNotification: \(obj.object as? NSObject)")
                    if let textField = obj.object as? UITextField {
                        textField.selectAll(nil)
                    }
                }
                #endif
                .focused($isURLBarFocused)
                .onChange(of: isURLBarFocused) { oldValue, newValue in
                    if newValue {
                        // whenever we focus the URL bar, we also restore the bottom bar
                        showBottomBar = true
                    }
                }
                #endif
                .onSubmit(of: .text) {
                    self.submitURL(viewModel.urlTextField)
                }
                .task(id: viewModel.urlTextField) {
                    if searchSuggestions && isURLBarFocused && !viewModel.urlTextField.isEmpty {
                        // query the search suggestions server
                        logger.log("querying search suggestions for: \(viewModel.urlTextField)")
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
                            #if !SKIP
                            .symbolRenderingMode(.hierarchical)
                            #endif
                    })
                    .buttonStyle(.plain)
                } else if self.state.isLoading {
                    Button(action: { self.viewModel.navigator.stopLoading() }, label: {
                        Image("xmark", bundle: .module)
                            #if !SKIP
                            .symbolRenderingMode(.hierarchical)
                            #endif
                    })
                    .buttonStyle(.plain)
                } else {
                    Button(action: { self.viewModel.navigator.reload() }, label: {
                        Image("arrow.clockwise", bundle: .module)
                            #if !SKIP
                            .symbolRenderingMode(.hierarchical)
                            #endif
                    })
                    .buttonStyle(.plain)
                    .frame(width: showBottomBar ? nil : 0.0) // hide button when the bottom bar is hidden
                    .opacity(showBottomBar ? 1.0 : 0.0) // hide button when the bottom bar is hidden
                }
            }
            .padding(showBottomBar ? 4.0 : 0.0)
        }
        .padding(.horizontal, showBottomBar ? 6.0 : 0.0)
        .background(showBottomBar ? Color(white: 0.75, opacity: 0.1) : Color.clear)
        #if !os(Android) // messes up the URL bar and causes it to be unresponsive
        .ignoresSafeArea([.container])
        #endif
    }

    @ViewBuilder func suggestionsView() -> some View {
        VStack {
            HStack {
                Button(action: {
                    withAnimation {
                        showSettings = true
                    }
                }, label: {
                    Image("gearshape.circle.fill", bundle: .module)
                        .resizable()
                        .foregroundStyle(.gray)
                        #if !SKIP
                        .symbolRenderingMode(.hierarchical)
                        #endif
                        .frame(width: 40, height: 40, alignment: .center)
                        .padding()
                })

                Spacer()
                TitleView()
                Spacer()
                Button(action: {
                    // de-focuses the text view and hides the URL bar
                    withAnimation {
                        self.isURLBarFocused = false
                    }
                }, label: {
                    Image("xmark.circle.fill", bundle: .module)
                        .resizable()
                        .foregroundStyle(.gray)
                        #if !SKIP
                        .symbolRenderingMode(.hierarchical)
                        #endif
                        .frame(width: 40, height: 40, alignment: .center)
                        .padding()
                })
            }

            Spacer()

            if let suggestions = currentSuggestions {
                List {
                    Section(suggestions.engine.name() + ": " + (suggestions.suggestions.isEmpty ? "No Suggestions" : "\(suggestions.suggestions.count) Suggestions")) {
                        ForEach(Array(suggestions.suggestions.enumerated()), id: \.0) { (index, suggestion) in
                            Button {
                                withAnimation {
                                    self.submitURL(suggestion)
                                    self.isURLBarFocused = false // close
                                }
                            } label: {
                                Text(suggestion)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    #if !SKIP
                                    .contentShape(Rectangle()) // needed to make the tap target fill the area
                                    #endif
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            } else {
                // TODO: show favorites
                Text("Favorites")
                    .font(.title)
                    .frame(maxHeight: .infinity)
            }
        }
        #if !SKIP
        .background(Color(UIColor.systemBackground))
        #endif
        //.background(Color.white)
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
                // TODO: update pre-existing history if the URL aleady exists
                _ = try store.saveItems(type: .history, items: [PageInfo(url: url, title: title)])
            }
        }
    }
}
#endif
