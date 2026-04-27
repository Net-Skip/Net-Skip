// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
import SkipWeb
import NetSkipModel

#if SKIP || os(iOS)

let fallbackURL = "about:blank"

///// the background for the
#if SKIP
let urlBarBackground = Color.clear
#else
let urlBarBackground = Color(uiColor: UIColor.secondarySystemBackground)
#endif
//
//let bottomBarBackground = Color.clear

/// The list of tabs that holds a BrowserView
@MainActor public struct BrowserTabView : View {
    let configuration: WebEngineConfiguration
    let store: WebBrowserStore

    @State var tabs: [BrowserViewModel] = []
    @State var selectedTab: PageInfo.ID = PageInfo.ID(0)

    @State var showBottomBar = true
    @State var showActiveTabs = false
    @State var showSettings = false
    @State var showHistory = false
    @State var showFavorites = false
    @State var showHistoryFavorites = false
    @State var historyFavoriesSelection = 1

    @State var triggerImpact = false
    @State var triggerWarning = false
    @State var triggerError = false
    @State var triggerStart = false
    @State var triggerStop = false

    @AppStorage("appearance") var appearance: String = "" // system
    @AppStorage("buttonHaptics") var buttonHaptics: Bool = true
    @AppStorage("pageLoadHaptics") var pageLoadHaptics: Bool = false
    @AppStorage("searchEngine") var searchEngine: SearchEngine.ID = ""
    @AppStorage("searchSuggestions") var searchSuggestions: Bool = true
    @AppStorage("userAgent") var userAgent: String = ""
    @AppStorage("blockAds") var blockAds: Bool = true
    @AppStorage("enableJavaScript") var enableJavaScript: Bool = true
    @AppStorage("requestDesktopSite") var requestDesktopSite: Bool = false
    @AppStorage("textZoom") var textZoom: Double = 1.0
    @AppStorage("selectedTabState") var selectedTabState: String = "" // app storage does not support Int64 (PageInfo.ID), so we serialize it as a string

    public init(configuration: WebEngineConfiguration, store: WebBrowserStore) {
        self.configuration = configuration
        self.store = store
    }

    public var body: some View {
        browserTabView()
            .toolbar {
                ToolbarItemGroup(placement: toolbarPlacement) {
                    backButton()
                    Spacer()
                    forwardButton()
                    Spacer()
                    shareButton()
                    Spacer()
                    showHistoryFavoritesButton()
                    Spacer()
                    tabsButton()
                }
            }
        .background(Color.clear)
        .toolbarBackground(Color.white.opacity(0.5), for: .bottomBar)
        .toolbar(showBottomBar ? .visible : .hidden, for: .bottomBar)
        .sheet(isPresented: $showSettings) { settingsView() }
        .sheet(isPresented: $showHistoryFavorites) { historyFavoritesPageInfoTabView() }
        .sheet(isPresented: $showHistory) { historyPageInfoView() }
        .sheet(isPresented: $showFavorites) { favoritesPageInfoView() }
        .sheet(isPresented: $showActiveTabs) { activeTabsView() }
        .preferredColorScheme(appearance == "dark" ? .dark : appearance == "light" ? .light : nil)
    }

    var toolbarPlacement: ToolbarItemPlacement {
        #if os(macOS)
        let toolbarPlacement = ToolbarItemPlacement.automatic
        #else
        let toolbarPlacement = ToolbarItemPlacement.bottomBar
        #endif
        return toolbarPlacement
    }

    func browserTabView() -> some View {
        /// We need to wrap a separate BrowserView because @AppStorage does not trigger `onChange` events, which we need to synchronize between the browser state and the preferences
        TabView(selection: $selectedTab) {
            ForEach($tabs) { tab in
                BrowserView(configuration: configuration, store: store, submitURL: { self.submitURL(text: $0) }, viewModel: tab, searchEngine: $searchEngine, searchSuggestions: $searchSuggestions, showSettings: $showSettings, showBottomBar: $showBottomBar, userAgent: $userAgent, blockAds: $blockAds, enableJavaScript: $enableJavaScript, pageLoadHaptics: $pageLoadHaptics, requestDesktopSite: $requestDesktopSite, textZoom: $textZoom)
            }
        }
        //.toolbarBackground(Color.clear, for: .bottomBar)
        //.toolbarBackground(.visible, for: .bottomBar) // needed to make toolbarBackground with color show up
        .background(LinearGradient(colors: [currentState?.themeColor ?? Color.clear, Color.clear], startPoint: .top, endPoint: .center)) // changes the status bar and toolbar
        #if !SKIP
        .tabViewStyle(PageTabViewStyle(indexDisplayMode: .never))
        .indexViewStyle(PageIndexViewStyle(backgroundDisplayMode: .automatic))
        .onOpenURL { url in
            openURL(url: url.absoluteString, newTab: true)
        }
        .sensoryFeedback(.start, trigger: triggerStart)
        .sensoryFeedback(.stop, trigger: triggerStop)
        .sensoryFeedback(.impact, trigger: triggerImpact)
        .sensoryFeedback(.warning, trigger: triggerWarning)
        .sensoryFeedback(.error, trigger: triggerError)
        #endif
        .onChange(of: currentState?.scrollingDown) {
            let scrollingDown = currentState?.scrollingDown == false
            // whenever we change scrolling, show/hide the bottom bar depending on the direction
            if self.showBottomBar != scrollingDown {
                withAnimation {
                    //logger.log("scrollingDown: \(scrollingDown) offset: \(currentState?.scrollingOffset ?? 0.0)")
                    self.showBottomBar = scrollingDown
                }
            }
        }
        .onChange(of: selectedTab) { oldTab, newTab in
            // Capture snapshot of the tab being left
            if let oldVM = tabs.first(where: { $0.id == oldTab }) {
                if oldVM.state.url != nil {
                    captureTabSnapshot(tab: oldVM)
                }
            }
            self.showBottomBar = true
            self.selectedTabState = newTab.description
        }
        .onAppear {
            logger.info("restoring active tabs")
            restoreActiveTabs()
            self.showBottomBar = true
        }
    }

    func settingsView() -> some View {
        SettingsView(configuration: configuration, store: store, appearance: $appearance, buttonHaptics: $buttonHaptics, pageLoadHaptics: $pageLoadHaptics, searchEngine: $searchEngine, searchSuggestions: $searchSuggestions, userAgent: $userAgent, blockAds: $blockAds, enableJavaScript: $enableJavaScript)
            #if !SKIP
            .environment(\.openURL, openURLAction(newTab: true))
            #endif
    }

    func restoreActiveTabs() {
        // restore the saved tabs
        let activeTabs = trying { try store.loadItems(type: PageInfo.PageType.active, ids: []) }

        var hasBlankTab = false
        var blankTabIdsToRemove: [PageInfo.ID] = []

        for activeTab in activeTabs ?? [] {
            let isBlank = activeTab.url == nil || activeTab.url == "" || activeTab.url == "about:blank"

            // Only keep one blank tab; remove duplicates from the database
            if isBlank {
                if hasBlankTab {
                    blankTabIdsToRemove.append(activeTab.id)
                    continue
                }
                hasBlankTab = true
            }

            logger.log("restoring tab \(activeTab.id): \(activeTab.url ?? "NONE") title=\(activeTab.title ?? "")")
            let viewModel = newViewModel(activeTab)
            self.tabs.append(viewModel)
        }

        // Clean up duplicate blank tabs from the database
        if !blankTabIdsToRemove.isEmpty {
            logger.info("removing \(blankTabIdsToRemove.count) duplicate blank tabs")
            try? store.removeItems(type: .active, ids: Set(blankTabIdsToRemove))
        }

        // always have at least one tab open
        if self.tabs.isEmpty {
            newTabAction()
        }

        // select the most recently selected tab if it exists
        if let selectedTabID = PageInfo.ID(selectedTabState), self.tabs.contains(where: { $0.id == selectedTabID }) {
            self.selectedTab = selectedTabID
        } else {
            self.selectedTab = tabs.last?.id ?? self.selectedTab
        }
    }

    func newViewModel(_ pageInfo: PageInfo) -> BrowserViewModel {
        let newID = (try? store.saveItems(type: .active, items: [pageInfo]).first) ?? PageInfo.ID(0)
        let newURL = URL(string: pageInfo.url ?? fallbackURL)
        return BrowserViewModel(id: newID, navigator: WebViewNavigator(initialURL: newURL), configuration: configuration, store: store)
    }

    // MARK: - Tab Snapshots

    func snapshotDirectory() -> URL {
        let dir = URL.cachesDirectory.appendingPathComponent("tab-snapshots")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    func snapshotPath(for tabId: PageInfo.ID) -> URL {
        return snapshotDirectory().appendingPathComponent("\(tabId).png")
    }

    func captureTabSnapshot(tab: BrowserViewModel) {
        guard let webEngine = tab.navigator.webEngine else { return }
        let tabId = tab.id
        Task { @MainActor in
            do {
                let config = SkipWebSnapshotConfiguration(snapshotWidth: 300)
                let snapshot = try await webEngine.takeSnapshot(configuration: config)
                let path = snapshotPath(for: tabId)
                try snapshot.pngData.write(to: path)
            } catch {
                logger.warning("Failed to capture tab snapshot: \(error)")
            }
        }
    }

    func captureAllTabSnapshots() {
        for tab in tabs {
            if tab.state.url != nil {
                captureTabSnapshot(tab: tab)
            }
        }
    }

    func removeTabSnapshot(tabId: PageInfo.ID) {
        let path = snapshotPath(for: tabId)
        try? FileManager.default.removeItem(at: path)
    }

    func loadSnapshotImage(for tabId: PageInfo.ID) -> UIImage? {
        let path = snapshotPath(for: tabId)
        guard let data = try? Data(contentsOf: path) else { return nil }
        return UIImage(data: data)
    }

    // MARK: - Tab Overview

    func activeTabsView() -> some View {
        NavigationStack {
            ScrollView {
                LazyVGrid(columns: [GridItem(.adaptive(minimum: 160), spacing: 12)], spacing: 12) {
                    ForEach(tabs) { tab in
                        tabCardView(tab: tab)
                    }
                }
                .padding(12)
            }
            .background(Color(white: 0.12))
            .navigationTitle(Text("\(tabs.count) Tabs", bundle: .module, comment: "tab count title"))
            #if !SKIP
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(action: {
                        newTabAction()
                        showActiveTabs = false
                    }) {
                        Image("plus", bundle: .module)
                    }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        showActiveTabs = false
                    } label: {
                        Text("Done", bundle: .module, comment: "done button title")
                            .bold()
                    }
                }
            }
        }
        #if !SKIP
        .presentationDetents([.medium, .large])
        #endif
    }

    @ViewBuilder func tabCardView(tab: BrowserViewModel) -> some View {
        let isActive = tab.id == selectedTab
        let title = tab.state.pageTitle ?? ""
        let urlString = tab.state.url?.absoluteString ?? ""
        let domain = tabDomainFromURL(urlString)
        let snapshotImage = loadSnapshotImage(for: tab.id)

        Button {
            withAnimation {
                self.selectedTab = tab.id
                self.showActiveTabs = false
            }
        } label: {
            VStack(alignment: .leading, spacing: 0) {
                // Title bar
                HStack(spacing: 4) {
                    if !urlString.isEmpty {
                        Image("lock", bundle: .module)
                            .font(.system(size: 9))
                            .foregroundStyle(Color.white.opacity(0.7))
                    }
                    Text(title.isEmpty ? (domain.isEmpty ? "New Tab" : domain) : title)
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(Color.white)
                        .lineLimit(1)
                    Spacer(minLength: 4)
                    // Close button
                    Button {
                        removeTabSnapshot(tabId: tab.id)
                        closeTabs([tab.id])
                    } label: {
                        Image("xmark", bundle: .module)
                            .font(.system(size: 10, weight: .bold))
                            .foregroundStyle(Color.white.opacity(0.6))
                    }
                    .buttonStyle(.plain)
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
                .background(isActive ? Color.accentColor : Color(white: 0.28))

                // Snapshot preview area
                ZStack {
                    Color(white: 0.95)
                    if let snapshotImage = snapshotImage {
                        Image(uiImage: snapshotImage)
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                    } else {
                        // Fallback: show domain text
                        VStack(spacing: 8) {
                            Image("magnifyingglass", bundle: .module)
                                .font(.system(size: 24))
                                .foregroundStyle(Color.gray.opacity(0.4))
                            if !domain.isEmpty {
                                Text(domain)
                                    .font(.system(size: 13))
                                    .foregroundStyle(Color.gray.opacity(0.6))
                            }
                        }
                    }
                }
                .frame(height: 180)
                .clipped()
            }
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .strokeBorder(isActive ? Color.accentColor : Color(white: 0.3), lineWidth: isActive ? 2.5 : 0.5)
            )
            .shadow(color: .black.opacity(0.3), radius: 3, y: 2)
        }
        .buttonStyle(.plain)
    }

    func tabDomainFromURL(_ urlString: String) -> String {
        guard let url = URL(string: urlString) else { return urlString }
        var host = url.host ?? ""
        if host.hasPrefix("www.") {
            host = host.replacingOccurrences(of: "www.", with: "")
        }
        return host
    }

    #if !SKIP
    func openURLAction(newTab: Bool) -> OpenURLAction {
        OpenURLAction(handler: { url in
            openURL(url: url.absoluteString, newTab: newTab)
            // TODO: reject unsupported URLs
            return OpenURLAction.Result.handled
        })
    }
    #endif

    func openURL(url: String, newTab: Bool) {
        logger.log("openURL: \(url) newTab=\(newTab)")
        // if we have no open tabs, of if the current tab is not blank, then open it in a new URL
        if self.currentViewModel == nil || (newTab == true && self.currentURL == nil) {
            newTabAction(url: url) // this will set the current tab
        }
        var newURL = url
        // if the scheme netskip:// then change it to https://
        if url.hasPrefix("netskip://") {
            newURL = url.replacingOccurrences(of: "netskip://", with: "https://")
        }
        if let navURL = URL(string: newURL) {
            currentNavigator?.load(url: navURL)
        }
    }

    var currentViewModel: BrowserViewModel? {
        tabs.first(where: { $0.id == self.selectedTab })
            ?? tabs.last
    }

    var currentState: WebViewState? {
        currentViewModel?.state
    }

    var currentNavigator: WebViewNavigator? {
        currentViewModel?.navigator
    }

    var currentWebView: PlatformWebView? {
        currentNavigator?.webEngine?.webView
    }

    var currentURL: String? {
        if let url = currentState?.pageURL {
            return url
        }

        if let url = currentWebView?.url {
            #if SKIP
            return url
            #else
            return url.absoluteString
            #endif
        }

        return nil
    }


    func historyFavoritesPageInfoTabView() -> some View {
        TabView(selection: $historyFavoriesSelection,
                content:  {
            NavigationStack {
                pageInfoPicker()
                historyPageInfoView()
            }
            .tag(1)
            NavigationStack {
                pageInfoPicker()
                favoritesPageInfoView()
            }
            .tag(2)
        })
        #if !SKIP
        .presentationDetents([.medium, .large])
        .presentationDragIndicator(.hidden)
        #endif
    }

    func pageInfoPicker() -> some View {
        Picker(selection: $historyFavoriesSelection) {
            Text("History", bundle: .module, comment: "tab selection for viewing the history list")
                .tag(1)
            Text("Favorites", bundle: .module, comment: "tab selection for viewing the favorites list")
                .tag(2)
        } label: {
            EmptyView()
        }
        #if !SKIP
        .pickerStyle(.segmented)
        #endif
    }

    func favoritesPageInfoView() -> some View {
        PageInfoListView(type: PageInfo.PageType.favorite, store: store, onSelect: { pageInfo in
            logger.info("select favorite: \(pageInfo.url ?? "NONE")")
            if let url = pageInfo.url {
                openURL(url: url, newTab: true)
            }
        }, onDelete: { pageInfos in
            //logger.info("delete histories: \(pageInfos.map(\.url))")
        }, toolbarItems: {
            ToolbarItem(placement: .automatic) {
                Button {
                    showFavorites = false
                    showHistoryFavorites = false
                } label: {
                    Text("Done", bundle: .module, comment: "done button title")
                        .bold()
                }
            }
        })
    }

    func historyPageInfoView() -> some View {
        PageInfoListView(type: PageInfo.PageType.history, store: store, onSelect: { pageInfo in
            logger.info("select history: \(pageInfo.url ?? "NONE")")
            if let url = pageInfo.url {
                openURL(url: url, newTab: true)
            }
        }, onDelete: { pageInfos in
            //logger.info("delete histories: \(pageInfos.map(\.url))")
        }, toolbarItems: {
            ToolbarItem(placement: .automatic) {
                Button {
                    showHistory = false
                    showHistoryFavorites = false
                } label: {
                    Text("Done", bundle: .module, comment: "done button title")
                        .bold()
                }
            }
        })
    }

    @ViewBuilder func backButton() -> some View {
        let enabled = currentState?.canGoBack == true
        let backLabel = Label {
            Text("Back", bundle: .module, comment: "back button label")
        } icon: {
            Image("chevron.left", bundle: .module)
        }

        if isSkip || !enabled {
            Button {
                backAction()
            } label: {
                backLabel
            }
            .disabled(!enabled)
        } else {
            Menu {
                backHistoryMenu()
            } label: {
                backLabel
            } primaryAction: {
                backAction()
            }
            .disabled(!enabled)
            .accessibilityIdentifier("button.back")
        }
    }

    @ViewBuilder func forwardButton() -> some View {
        let enabled = currentState?.canGoForward == true
        let forwardLabel = Label {
            Text("Forward", bundle: .module, comment: "forward button label")
        } icon: {
            Image("chevron.right", bundle: .module)
        }

        if isSkip || !enabled {
            Button {
                forwardAction()
            } label: {
                forwardLabel
            }
            .disabled(!enabled)
        } else {
            Menu {
                forwardHistoryMenu()
            } label: {
                forwardLabel
            } primaryAction: {
                forwardAction()
            }
            .disabled(!enabled)
            .accessibilityIdentifier("button.forward")
        }
    }

    @ViewBuilder func shareButton() -> some View {
        ShareLink(item: currentState?.pageURL ?? fallbackURL) {
            Label {
                Text("Share", bundle: .module, comment: "share button label")
            } icon: {
                Image("ios_share", bundle: .module)
            }
        }
        .disabled(currentState?.pageURL == nil)
        .accessibilityIdentifier("button.share")
    }

    @ViewBuilder func showHistoryFavoritesButton() -> some View {
        Button(action: showHistoryFavoritesAction) {
            Label {
                Text("Bookmarks", bundle: .module, comment: "bookmarks button label")
            } icon: {
                Image("book", bundle: .module)
            }
        }
        .accessibilityIdentifier("button.history.favorites")
    }

    @ViewBuilder func tabsButton() -> some View {
        Menu {
            Button(action: { newTabAction() }) {
                Label {
                    Text("New Tab", bundle: .module, comment: "more button string for creating a new tab")
                } icon: {
                    Image("plus", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.button.newtab")

            Button(action: closeTabAction) {
                Label {
                    Text("Close Tab", bundle: .module, comment: "more button string for closing a tab")
                } icon: {
                    Image("xmark", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.button.close")

            Divider()

            Button(action: reloadAction) {
                Label {
                    Text("Reload", bundle: .module, comment: "reload button label")
                } icon: {
                    Image("arrow.clockwise", bundle: .module)
                }
            }

            Button(action: homeAction) {
                Label {
                    Text("Home", bundle: .module, comment: "home button label")
                } icon: {
                    Image("house", bundle: .module)
                }
            }

            Button(action: findOnPageAction) {
                Label {
                    Text("Find on Page", bundle: .module, comment: "find on page button label")
                } icon: {
                    Image("magnifyingglass", bundle: .module)
                }
            }

            Button(action: favoriteAction) {
                Label {
                    Text("Add to Favorites", bundle: .module, comment: "add to favorites button label")
                } icon: {
                    Image("star", bundle: .module)
                }
            }

            Divider()

            Button(action: zoomInAction) {
                Label {
                    Text("Zoom In", bundle: .module, comment: "zoom in button label")
                } icon: {
                    Image("zoom_in", bundle: .module)
                }
            }

            Button(action: zoomOutAction) {
                Label {
                    Text("Zoom Out", bundle: .module, comment: "zoom out button label")
                } icon: {
                    Image("zoom_out", bundle: .module)
                }
            }

            Button(action: toggleDesktopSiteAction) {
                Label {
                    Text(requestDesktopSite ? "Mobile Site" : "Desktop Site", bundle: .module, comment: "desktop/mobile site toggle button label")
                } icon: {
                    Image(requestDesktopSite ? "smartphone" : "computer", bundle: .module)
                }
            }

            Divider()

            Button(action: settingsAction) {
                Label {
                    Text("Settings", bundle: .module, comment: "settings button label")
                } icon: {
                    Image("gearshape", bundle: .module)
                }
            }
        } label: {
            Label {
                Text("Tabs", bundle: .module, comment: "tabs button label")
            } icon: {
                tabCountIcon()
            }
        } primaryAction: {
            tabListAction()
        }
        .accessibilityIdentifier("button.tabs")
    }

    @ViewBuilder func tabCountIcon() -> some View {
        ZStack {
            RoundedRectangle(cornerRadius: 4)
                .strokeBorder(lineWidth: 1.5)
                .frame(width: 22, height: 22)
            Text("\(tabs.count)")
                .font(.system(size: 11, weight: .bold))
        }
    }

    @ViewBuilder func backHistoryMenu() -> some View {
        ForEach(Array((currentState?.backList ?? []).enumerated()), id: \.0) {
            historyItem(item: $0.1)
        }
    }

    @ViewBuilder func forwardHistoryMenu() -> some View {
        ForEach(Array((currentState?.forwardList ?? []).enumerated()), id: \.0) {
            historyItem(item: $0.1)
        }
    }

    @ViewBuilder func historyItem(item: WebHistoryItem) -> some View {
        Button(item.title?.isEmpty == false ? (item.title ?? "") : item.url) {
            currentViewModel?.navigator.go(item)
        }
    }

    @MainActor func submitURL(text: String) {
        logger.log("URLBar submit")
        if let url = fieldToURL(text),
            ["http", "https", "file", "ftp", "netskip"].contains(url.scheme ?? "") {
            logger.log("loading url: \(url)")
            self.currentNavigator?.load(url: url)
        } else {
            logger.log("URL search bar entry: \(text)")
            if let searchEngine = SearchEngine.lookup(id: self.searchEngine),
               let queryURL = searchEngine.queryURL(text, Locale.current.identifier) {
                logger.log("search engine query URL: \(queryURL)")
                if let url = URL(string: queryURL) {
                    self.currentNavigator?.load(url: url)
                }
            }
        }
    }

    /// The home page URL, which default to the current search engine's home page
    var homeURL: URL? {
        if let homePage = SearchEngine.lookup(id: self.searchEngine)?.homeURL,
           let homePageURL = URL(string: homePage) {
            return homePageURL
        } else {
            return nil
        }
    }

    private func fieldToURL(_ string: String) -> URL? {
        if string.hasPrefix("https://")
            || string.hasPrefix("http://")
            || string.hasPrefix("file://") {
            return URL(string: string)
        } else if string.contains(" ") {
            // anything with spaces is probably a search term
            return nil
        } else if string.contains(".") {
            // anything with a dot might be a URL (TODO: check domain suffix?)
            // fall back to "https" as the protocol for a bare URL string like "appfair.net"
            let url = URL(string: string)
            if url?.scheme == nil {
                // a URL with no scheme should default to https
                return URL(string: "https://\(string)")
            } else {
                return url
            }
        } else {
            return nil
        }
    }


    func hapticFeedback() {
        #if !SKIP
        if buttonHaptics {
            triggerImpact.toggle()
        }
        #endif
    }

    func homeAction() {
        logger.info("homeAction")
        hapticFeedback()
        if let homeURL = homeURL {
            currentNavigator?.load(url: homeURL)
        }
    }

    func backAction() {
        logger.info("backAction")
        hapticFeedback()
        currentNavigator?.goBack()
    }

    func forwardAction() {
        logger.info("forwardAction")
        hapticFeedback()
        currentNavigator?.goForward()
    }

    func reloadAction() {
        logger.info("reloadAction")
        hapticFeedback()
        currentNavigator?.reload()
    }

    func closeTabAction() {
        logger.info("closeTabAction")
        hapticFeedback()
        closeTabs([self.selectedTab])
    }

    func closeTabs(_ ids: Set<PageInfo.ID>) {
        for id in ids {
            removeTabSnapshot(tabId: id)
        }
        withAnimation {
            self.tabs.removeAll(where: { ids.contains($0.id) })
            // remove the pages from the tab list
            try? store.removeItems(type: .active, ids: ids)
            self.selectedTab = self.tabs.last?.id ?? self.selectedTab

            // always leave behind a single tab
            if self.tabs.isEmpty {
                newTabAction()
            }
        }
        logTabs()
    }

    func findOnPageAction() {
        logger.info("findOnPageAction")
        hapticFeedback()
        #if !SKIP
        if let interaction = currentWebView?.findInteraction {
            interaction.presentFindNavigator(showingReplace: false)
        }
        #endif
    }

    func newTabAction(url: String? = nil) {
        logger.info("newTabAction url=\(url ?? "nil")")
        hapticFeedback()

        // If requesting a blank tab, reuse an existing blank tab instead of creating another
        if url == nil {
            for tab in tabs {
                if tab.state.url == nil && (tab.state.pageURL == nil || tab.state.pageURL == "about:blank") {
                    self.selectedTab = tab.id
                    logTabs()
                    return
                }
            }
        }

        let info = PageInfo(url: url)
        let vm = newViewModel(info)
        self.tabs.append(vm)
        self.selectedTab = vm.id
        logTabs()
    }

    func logTabs() {
        logger.info("selected=\(selectedTab) count=\(self.tabs.count)") //  tabs=\(self.tabs.flatMap(\.navigator.webEngine?.webView.url))")
    }

    func newPrivateTabAction() {
        logger.info("newPrivateTabAction")
        hapticFeedback()
        // TODO
    }

    func tabListAction() {
        logger.info("tabListAction")
        hapticFeedback()
        captureAllTabSnapshots()
        self.showActiveTabs = true
    }

    func favoriteAction() {
        logger.info("favoriteAction")
        hapticFeedback()
        if let url = self.currentURL {
            logger.info("addPageToFavorite: \(url)")
            trying {
                _ = try store.saveItems(type: .favorite, items: [PageInfo(url: url, title: currentState?.pageTitle ?? currentWebView?.title)])
            }
        }
    }

    func showHistoryFavoritesAction() {
        logger.info("showHistoryFavoritesAction")
        hapticFeedback()
        showHistoryFavorites = true
    }

    func historyAction() {
        logger.info("historyAction")
        hapticFeedback()
        showHistory = true
    }

    func favoritesAction() {
        logger.info("favoritesAction")
        hapticFeedback()
        showFavorites = true
    }

    func settingsAction() {
        logger.info("settingsAction")
        hapticFeedback()
        showSettings = true
    }

    func zoomInAction() {
        logger.info("zoomInAction")
        hapticFeedback()
        textZoom = min(textZoom + 0.1, 3.0)
    }

    func zoomOutAction() {
        logger.info("zoomOutAction")
        hapticFeedback()
        textZoom = max(textZoom - 0.1, 0.5)
    }

    func toggleDesktopSiteAction() {
        logger.info("toggleDesktopSiteAction: \(!requestDesktopSite)")
        hapticFeedback()
        requestDesktopSite.toggle()
        currentNavigator?.reload()
    }

}

struct TitleView : View {
    var body: some View {
        Text("Net Skip", bundle: .module, comment: "title screen headline")
            .font(.system(size: 28, weight: .bold))
            .foregroundStyle(.primary)
    }
}



@Observable public class BrowserViewModel: Identifiable {
    /// The persistent ID of the page
    // SKIP INSERT: @Suppress("MUST_BE_INITIALIZED_OR_FINAL_OR_ABSTRACT") // var to workaround Kotlin to 2 error: ”Property must be initialized, be final, or be abstract.”
    public let id: PageInfo.ID
    let navigator: WebViewNavigator
    let configuration: WebEngineConfiguration
    let store: WebBrowserStore
    var state = WebViewState()
    var urlTextField = ""

    public init(id: PageInfo.ID, navigator: WebViewNavigator, configuration: WebEngineConfiguration, store: WebBrowserStore) {
        self.id = id
        self.navigator = navigator
        self.configuration = configuration
        self.store = store
    }
}

#endif

