// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
#if !SKIP
import UIKit
#endif
import SkipWeb
import NetSkipModel
import NetSkipMiniApp

#if SKIP || os(iOS)

let fallbackURL = "about:blank"

/// Opaque chrome background — used by both the URL bar capsule and the
/// bottom toolbar so the WebView's content never bleeds through.
/// `secondarySystemBackground` adapts to dark mode on iOS; Skip's
/// translation of the same Color picks up Material's secondary surface.
#if SKIP
let urlBarBackground = Color(white: 0.92)
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
    @State var showDownloads = false
    @State var historyFavoriesSelection = 1
    @State var showFindBar = false
    @State var showPageZoom = false
    @State var findText = ""
    @State var tabSearchText = ""

    @State var triggerImpact = false
    @State var triggerWarning = false
    @State var triggerError = false
    @State var triggerStart = false
    @State var triggerStop = false

    @Environment(NetSkipSettings.self) var settings

    /// In-flight per-tab UI state (not a user preference) — keep on AppStorage.
    @AppStorage("selectedTabState") var selectedTabState: String = ""

    @State var tabsSegment: Int = 1 // 1 = Pages, 2 = Apps
    @State var runningMiniApps: Set<String> = [] // IDs of launched miniapps
    @State var activeMiniAppItem: MiniAppLaunchItem? = nil // app storage does not support Int64 (PageInfo.ID), so we serialize it as a string

    /// Stack of recently-closed-tab URLs. Most-recent first. Capped so the
    /// "Reopen Closed Tab" menu doesn't grow unbounded in long sessions.
    /// Per-session only — we explicitly don't persist these because the
    /// user's expectation is that this is undo-style, not "history".
    @State var recentlyClosedTabURLs: [String] = []
    private static let recentlyClosedTabsLimit: Int = 10

    /// Height of the bottom chrome bar (back/tabs/menu/bookmarks/forward
    /// HStack). Shared with `BrowserView` so the URL bar can pad itself
    /// up by exactly this many points and sit flush against the toolbar.
    static let bottomToolbarHeight: CGFloat = 48.0

    /// Natural height of the URL bar capsule when shown — matches the
    /// 44pt capsule + 4pt top padding inside `urlBarComponentView` so
    /// there's no centered-frame gap on Compose. Collapses to zero on
    /// scroll-down so the WebView occupies the full screen.
    static let urlBarHeight: CGFloat = 48.0

    @State var confirmCloseAllTabs: Bool = false
    @State var isCurrentPageFavorited: Bool = false

    public init(configuration: WebEngineConfiguration, store: WebBrowserStore) {
        self.configuration = configuration
        self.store = store
    }

    public var body: some View {
        // SkipUI insets the top edge by the system status bar automatically on Android
        // (matching iOS), so no manual safe-area padding is needed here.
        bodyContent
    }

    @ViewBuilder
    private var bodyContent: some View {
        ZStack(alignment: .bottom) {
            // browserTabView fills the entire ZStack; the URL bar lives
            // inside each tab's `BrowserView` as a `.bottom`-aligned
            // overlay so the WebView extends behind it. We lift it by
            // `bottomToolbarHeight` so the URL bar sits flush against
            // the toolbar that we overlay separately below.
            VStack(spacing: 0) {
                browserTabView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                if showFindBar {
                    findBar()
                }
            }
            // Bottom toolbar overlaid at the bottom — translucent so the
            // WebView's content shows through, and collapsed to zero
            // when `showBottomBar` is off so it disappears entirely on
            // scroll-down.
            bottomToolbar()
            if showPageZoom {
                pageZoomBar()
            }
        }
        .background(Color.clear)
        .statusBarHidden(settings.hideStatusBar)
        // Edge-to-edge full-bleed only when the user opts in via
        // Hide Status Bar. `.all` here gives the WebView every edge of
        // the screen; the URL bar / toolbar overlays stay anchored
        // inside the (now ignored) safe area at the bottom.
        .ignoresSafeArea(edges: settings.hideStatusBar ? Edge.Set.all : Edge.Set(rawValue: 0))
        .sheet(isPresented: $showSettings) { settingsView() }
        .sheet(isPresented: $showHistoryFavorites) { historyFavoritesPageInfoTabView() }
        .sheet(isPresented: $showHistory) { historyPageInfoView() }
        .sheet(isPresented: $showFavorites) { favoritesPageInfoView() }
        .sheet(isPresented: $showActiveTabs) { activeTabsView() }
        .sheet(isPresented: $showDownloads) { NetSkipDownloadsListView() }
        .confirmationDialog(
            Text("Close all \(tabs.count) tabs?",
                 bundle: .module,
                 comment: "title for the confirmation dialog when the user has chosen to close every open tab; argument is the tab count"),
            isPresented: $confirmCloseAllTabs,
            titleVisibility: .visible
        ) {
            Button(role: .destructive) {
                performCloseAllTabs()
            } label: {
                Text("Close All Tabs", bundle: .module, comment: "destructive confirm button on the close-all-tabs dialog")
            }
            .accessibilityIdentifier("button.closeAllTabs.confirm")
            Button(role: .cancel) {
                confirmCloseAllTabs = false
            } label: {
                Text("Cancel", bundle: .module, comment: "cancel button on the close-all-tabs dialog")
            }
            .accessibilityIdentifier("button.closeAllTabs.cancel")
        }
        .onReceive(NotificationCenter.default.publisher(for: .netSkipDownloadEnqueued)) { _ in
            if !showDownloads {
                showDownloads = true
            }
        }
        .preferredColorScheme(settings.appearance == "dark" ? .dark : settings.appearance == "light" ? .light : nil)
        .onChange(of: settings.blockAds, initial: false) { _, _ in applyContentBlockingSettings() }
        .onChange(of: settings.blockTrackers, initial: false) { _, _ in applyContentBlockingSettings() }
        .onChange(of: settings.blockCookieBanners, initial: false) { _, _ in applyContentBlockingSettings() }
        .onChange(of: settings.contentBlockingWhitelistedDomains, initial: false) { _, _ in applyContentBlockingSettings() }
        .onChange(of: settings.contentBlockingCustomBlockedPatterns, initial: false) { _, _ in applyContentBlockingSettings() }
        .onChange(of: currentURL ?? "") { _, _ in refreshFavoritedStatus() }
    }

    /// Apply content-blocker settings to the shared `WebEngineConfiguration` and
    /// trigger an iOS rule-list reinstall on each open tab. The Android blocker
    /// reads `UserDefaults` dynamically, so its changes apply on the next request
    /// without further work here.
    func applyContentBlockingSettings() {
        configuration.contentBlockers = netSkipMakeContentBlockerConfiguration(provider: ContentView.adBlockProvider)
        #if !SKIP
        Task { @MainActor in
            for tab in tabs {
                if let engine = tab.navigator.webEngine {
                    _ = await engine.reapplyContentBlockers()
                }
            }
        }
        #endif
    }

    var toolbarPlacement: ToolbarItemPlacement {
        #if os(macOS)
        let toolbarPlacement = ToolbarItemPlacement.automatic
        #else
        let toolbarPlacement = ToolbarItemPlacement.bottomBar
        #endif
        return toolbarPlacement
    }

    /// Bottom toolbar rendered as a manual HStack so the chrome can sit
    /// directly under the URL bar (which lives inside `BrowserView`'s own
    /// VStack) and disappear in tandem with it. Replaces the previous
    /// `.toolbar(.bottomBar)` placement that required a `NavigationStack`
    /// wrapper. Collapses to zero height when `showBottomBar` is off so
    /// the URL bar's compact slim mode is the only thing left visible.
    @ViewBuilder func bottomToolbar() -> some View {
        HStack(spacing: 0) {
            backButton()
            Spacer()
            tabsButton()
            Spacer()
            ellipsisMenu()
            Spacer()
            showHistoryFavoritesButton()
            Spacer()
            forwardButton()
        }
        .labelStyle(.iconOnly)
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity)
        .background(urlBarBackground)
        // Fixed height when shown; collapses to 0 when the user scrolls
        // down. SwiftUI's natural-sized `nil` height doesn't transpile
        // cleanly to Compose, and `.infinity` greedily eats the
        // WebView's space on iOS — so we just pick a height that
        // matches the standard system bottom bar.
        .frame(height: showBottomBar ? 48.0 : 0.0)
        .opacity(showBottomBar ? 1.0 : 0.0)
        .clipped()
    }

    func browserTabView() -> some View {
        /// We need to wrap a separate BrowserView because @AppStorage does not trigger `onChange` events, which we need to synchronize between the browser state and the preferences
        TabView(selection: $selectedTab) {
            ForEach($tabs) { tab in
                BrowserView(configuration: configuration, store: store, submitURL: { self.submitURL(text: $0) }, viewModel: tab, showSettings: $showSettings, showBottomBar: $showBottomBar)
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
        SettingsView(configuration: configuration, store: store, onClearCache: clearWebCacheAction)
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
        let vm = BrowserViewModel(id: newID, navigator: WebViewNavigator(initialURL: newURL), configuration: configuration, store: store)
        vm.savedTitle = pageInfo.title ?? ""
        vm.savedURL = pageInfo.url ?? ""
        return vm
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
            VStack(spacing: 0) {
                if settings.enableMiniApps {
                    Picker(selection: $tabsSegment) {
                        Text("Pages", bundle: .module, comment: "tab segment for pages")
                            .tag(1)
                        Text("Apps", bundle: .module, comment: "tab segment for miniapps")
                            .tag(2)
                    } label: {
                        EmptyView()
                    }
                    .pickerStyle(.segmented)
                    .colorScheme(.dark)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                }

                if tabsSegment == 1 || !settings.enableMiniApps {
                    pagesTabContent
                } else {
                    miniAppsTabContent
                }
            }
            .background(Color(white: 0.12))
            .navigationTitle(Text(tabsSegment == 1 || !settings.enableMiniApps ? "\(tabs.count) Tabs" : "Mini Apps", bundle: .module, comment: "tabs title"))
            #if !SKIP
            .navigationBarTitleDisplayMode(.inline)
            .toolbarColorScheme(.dark, for: .navigationBar)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    if tabsSegment == 1 || !settings.enableMiniApps {
                        Button(action: {
                            newTabAction()
                            showActiveTabs = false
                        }) {
                            Image("plus", bundle: .module)
                        }
                        .accessibilityIdentifier("button.tabs.new")
                        .accessibilityLabel(Text("New Tab", bundle: .module, comment: "accessibility label for the new-tab toolbar button in the tab overview"))
                    }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        showActiveTabs = false
                    } label: {
                        Text("Done", bundle: .module, comment: "done button title")
                            .bold()
                    }
                    .accessibilityIdentifier("button.tabs.done")
                }
            }
        }
        #if !SKIP
        .presentationDetents([.large])
        #endif
    }

    var pagesTabContent: some View {
        VStack(spacing: 0) {
            tabSearchField

            let visibleTabs = filteredTabs
            if visibleTabs.isEmpty && !tabSearchText.isEmpty {
                VStack(spacing: 8) {
                    Spacer()
                    Image("magnifyingglass", bundle: .module)
                        .font(.system(size: 36))
                        .foregroundStyle(Color.white.opacity(0.4))
                    Text("No matching tabs", bundle: .module, comment: "empty-state message when the tab-overview search has no matches")
                        .font(.system(size: 14))
                        .foregroundStyle(Color.white.opacity(0.6))
                    Spacer()
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .accessibilityIdentifier("label.tabSearch.empty")
            } else {
                ScrollView {
                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 160), spacing: 12)], spacing: 12) {
                        ForEach(visibleTabs) { tab in
                            tabCardView(tab: tab)
                        }
                    }
                    .padding(12)
                }
            }
        }
    }

    /// Subset of `tabs` matching the current `tabSearchText`. Empty search
    /// passes everything through unchanged. The match is case-insensitive
    /// and falls back to the view-model's saved fields when the live
    /// WebView state hasn't been populated for a background tab — the
    /// same fallback chain the tab card uses for its display text.
    var filteredTabs: [BrowserViewModel] {
        let query = tabSearchText.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        if query.isEmpty { return tabs }
        return tabs.filter { tab in
            let titleSource = tab.state.pageTitle ?? tab.savedTitle
            let urlSource = tab.state.pageURL ?? tab.state.url?.absoluteString ?? tab.savedURL
            return titleSource.lowercased().contains(query) || urlSource.lowercased().contains(query)
        }
    }

    @ViewBuilder var tabSearchField: some View {
        HStack(spacing: 8) {
            Image("magnifyingglass", bundle: .module)
                .foregroundStyle(Color.white.opacity(0.6))
            TextField(text: $tabSearchText) {
                Text("Search Tabs", bundle: .module, comment: "placeholder text for the search field in the tab overview")
            }
            .textFieldStyle(.plain)
            .foregroundStyle(Color.white)
            #if !SKIP
            .autocorrectionDisabled(true)
            .textInputAutocapitalization(.never)
            #endif
            .accessibilityIdentifier("field.tabSearch")

            if !tabSearchText.isEmpty {
                Button {
                    tabSearchText = ""
                } label: {
                    Image("xmark.circle.fill", bundle: .module)
                        .foregroundStyle(Color.white.opacity(0.6))
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier("button.tabSearch.clear")
                .accessibilityLabel(Text("Clear search", bundle: .module, comment: "accessibility label for clearing the tab-overview search field"))
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color(white: 0.22))
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .padding(.horizontal, 12)
        .padding(.top, 8)
        .padding(.bottom, 4)
    }

    var miniAppsTabContent: some View {
        ScrollView {
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 160), spacing: 12)], spacing: 12) {
                ForEach(sampleMiniApps) { app in
                    miniAppCardView(app: app)
                }
            }
            .padding(12)
        }
        .sheet(item: $activeMiniAppItem) { item in
            miniAppSheet(appID: item.appID)
        }
    }

    @ViewBuilder func miniAppCardView(app: MiniAppCatalogEntry) -> some View {
        let isRunning = runningMiniApps.contains(app.id)
        let snapshotImage = loadMiniAppSnapshotImage(for: app.id)

        Button {
            activeMiniAppItem = MiniAppLaunchItem(appID: app.id)
            if !isRunning {
                runningMiniApps.insert(app.id)
            }
        } label: {
            VStack(alignment: .leading, spacing: 0) {
                // Title bar
                HStack(spacing: 4) {
                    Image("widgets", bundle: .module)
                        .font(.system(size: 10))
                        .foregroundStyle(Color.white.opacity(0.7))
                    Text(app.name)
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(Color.white)
                        .lineLimit(1)
                    Spacer(minLength: 4)
                    if isRunning {
                        Button {
                            closeMiniApp(id: app.id)
                        } label: {
                            Image("close", bundle: .module)
                                .font(.system(size: 10, weight: .bold))
                                .foregroundStyle(Color.white.opacity(0.6))
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
                .background(isRunning ? Color.accentColor : Color(white: 0.28))

                // Content area — show snapshot if available, otherwise placeholder
                ZStack {
                    Color(white: 0.95)
                    if let snapshotImage = snapshotImage {
                        Image(uiImage: snapshotImage)
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                    } else {
                        VStack(spacing: 6) {
                            Image("widgets", bundle: .module)
                                .font(.system(size: 28))
                                .foregroundStyle(isRunning ? Color.accentColor : Color.gray.opacity(0.4))
                            Text(app.description)
                                .font(.system(size: 11))
                                .foregroundStyle(Color.gray.opacity(0.7))
                                .multilineTextAlignment(.center)
                                .lineLimit(2)
                                .padding(.horizontal, 8)
                        }
                    }
                }
                .frame(height: 140)
                .clipped()
            }
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .strokeBorder(isRunning ? Color.accentColor : Color(white: 0.3), lineWidth: isRunning ? 2.0 : 0.5)
            )
            .shadow(color: .black.opacity(0.3), radius: 3, y: 2)
        }
        .buttonStyle(.plain)
    }

    func miniAppSheet(appID: String) -> some View {
        Group {
            if let entry = sampleMiniApps.first(where: { $0.id == appID }) {
                MiniAppHostingView(entry: entry, onDismiss: {
                    activeMiniAppItem = nil
                }, onSnapshot: { pngData in
                    saveMiniAppSnapshot(appID: appID, pngData: pngData)
                })
            }
        }
    }

    func saveMiniAppSnapshot(appID: String, pngData: Data) {
        let path = miniAppSnapshotPath(for: appID)
        do {
            try pngData.write(to: path)
        } catch {
            logger.warning("Failed to save miniapp snapshot: \(error)")
        }
    }

    func loadMiniAppSnapshotImage(for appID: String) -> UIImage? {
        let path = miniAppSnapshotPath(for: appID)
        guard let data = try? Data(contentsOf: path) else { return nil }
        return UIImage(data: data)
    }

    func closeMiniApp(id: String) {
        runningMiniApps.remove(id)
        // Clear the miniapp's persistent storage and snapshot
        let storageDir = miniAppStorageBaseDirectory.appendingPathComponent(id)
        try? FileManager.default.removeItem(at: storageDir)
        let snapshotFile = miniAppSnapshotPath(for: id)
        try? FileManager.default.removeItem(at: snapshotFile)
    }

    @ViewBuilder func tabCardView(tab: BrowserViewModel) -> some View {
        let isActive = tab.id == selectedTab
        let title = tab.state.pageTitle ?? tab.savedTitle
        let urlString = tab.state.url?.absoluteString ?? tab.savedURL
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
                    .accessibilityIdentifier("button.tab.close")
                    .accessibilityLabel(Text("Close tab", bundle: .module, comment: "accessibility label for closing an individual tab from the tab overview"))
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
        .contextMenu {
            Button(action: { copyURLForTab(tab) }) {
                Label {
                    Text("Copy URL", bundle: .module, comment: "context-menu item that copies a single tab's URL to the clipboard from the tab overview")
                } icon: {
                    Image("content_copy", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.tabCard.copyURL")
            .disabled((tab.state.pageURL ?? tab.savedURL).isEmpty)

            Button(role: .destructive, action: {
                removeTabSnapshot(tabId: tab.id)
                closeTabs([tab.id])
            }) {
                Label {
                    Text("Close Tab", bundle: .module, comment: "context-menu item that closes a single tab from the tab overview")
                } icon: {
                    Image("xmark", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.tabCard.close")
        }
    }

    /// Copies the URL of a specific tab to the system clipboard. Mirrors
    /// `copyURLAction` but acts on the passed-in tab rather than the
    /// currently selected one — needed for the tab-card context menu,
    /// where the user might right-click a background tab.
    func copyURLForTab(_ tab: BrowserViewModel) {
        let url = tab.state.pageURL ?? tab.savedURL
        guard !url.isEmpty else { return }
        logger.info("copyURLForTab id=\(tab.id) url=\(url)")
        hapticFeedback()
        #if SKIP
        let ctx = ProcessInfo.processInfo.androidContext
        let cm = ctx.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
        cm.setPrimaryClip(android.content.ClipData.newPlainText("URL", url))
        #else
        UIPasteboard.general.string = url
        #endif
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
        NavigationStack {
            VStack(spacing: 0) {
                Picker(selection: $historyFavoriesSelection) {
                    Text("History", bundle: .module, comment: "tab selection for viewing the history list")
                        .tag(1)
                    Text("Favorites", bundle: .module, comment: "tab selection for viewing the favorites list")
                        .tag(2)
                } label: {
                    EmptyView()
                }
                .pickerStyle(.segmented)
                .padding(.horizontal, 16)
                .padding(.vertical, 8)

                if historyFavoriesSelection == 1 {
                    historyPageInfoView()
                } else {
                    favoritesPageInfoView()
                }
            }
        }
        #if !SKIP
        .presentationDetents([.medium, .large])
        #endif
    }

    func favoritesPageInfoView() -> some View {
        NavigationStack { favoritesPageInfoListView() }
    }

    @ViewBuilder
    func favoritesPageInfoListView() -> some View {
        PageInfoListView(type: PageInfo.PageType.favorite, store: store, onSelect: { pageInfo in
            logger.info("select favorite: \(pageInfo.url ?? "NONE")")
            if let url = pageInfo.url {
                openURL(url: url, newTab: true)
            }
        }, onDelete: { pageInfos in
            //logger.info("delete histories: \(pageInfos.map(\.url))")
        }, onOpenInNewTab: { pageInfo in
            logger.info("openInNewTab favorite: \(pageInfo.url ?? "NONE")")
            if let url = pageInfo.url {
                newTabAction(url: url, inBackground: settings.openLinksInBackground)
            }
        }, onOpenAllInTabs: { pageInfos in
            logger.info("openAllInTabs favorites: count=\(pageInfos.count)")
            // Bulk-open backgrounds all but the last to mimic the
            // canonical "open all bookmarks" gesture in desktop browsers.
            for (i, pageInfo) in pageInfos.enumerated() {
                if let url = pageInfo.url {
                    let isLast = i == pageInfos.count - 1
                    newTabAction(url: url, inBackground: !isLast)
                }
            }
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
        NavigationStack { historyPageInfoListView() }
    }

    @ViewBuilder
    func historyPageInfoListView() -> some View {
        PageInfoListView(type: PageInfo.PageType.history, store: store, onSelect: { pageInfo in
            logger.info("select history: \(pageInfo.url ?? "NONE")")
            if let url = pageInfo.url {
                openURL(url: url, newTab: true)
            }
        }, onDelete: { pageInfos in
            //logger.info("delete histories: \(pageInfos.map(\.url))")
        }, onOpenInNewTab: { pageInfo in
            logger.info("openInNewTab history: \(pageInfo.url ?? "NONE")")
            if let url = pageInfo.url {
                newTabAction(url: url, inBackground: settings.openLinksInBackground)
            }
        }, onOpenAllInTabs: nil, toolbarItems: {
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

    let toolbarIconSize: CGFloat = 22.0

    @ViewBuilder func backButton() -> some View {
        let enabled = currentState?.canGoBack == true
        let backLabel = Label {
            Text("Back", bundle: .module, comment: "back button label")
        } icon: {
            Image("chevron.left", bundle: .module)
                .font(.system(size: toolbarIconSize))
        }

        // Tap fires the `primaryAction` (single-step back) on both platforms;
        // long-press opens the back-history menu so the user can jump
        // multiple steps. SkipWeb exposes `backList` on Android, and the
        // SkipUI accessibility-identifier-propagation fix means the
        // generated `DropdownMenuItem`s are addressable from UI tests.
        if !enabled {
            Button {
                backAction()
            } label: {
                backLabel
            }
            .disabled(true)
            .accessibilityIdentifier("button.back")
        } else {
            Menu {
                backHistoryMenu()
            } label: {
                backLabel
            } primaryAction: {
                backAction()
            }
            .accessibilityIdentifier("button.back")
        }
    }

    @ViewBuilder func forwardButton() -> some View {
        let enabled = currentState?.canGoForward == true
        let forwardLabel = Label {
            Text("Forward", bundle: .module, comment: "forward button label")
        } icon: {
            Image("chevron.right", bundle: .module)
                .font(.system(size: toolbarIconSize))
        }

        if !enabled {
            Button {
                forwardAction()
            } label: {
                forwardLabel
            }
            .disabled(true)
            .accessibilityIdentifier("button.forward")
        } else {
            Menu {
                forwardHistoryMenu()
            } label: {
                forwardLabel
            } primaryAction: {
                forwardAction()
            }
            .accessibilityIdentifier("button.forward")
        }
    }

    @ViewBuilder func showHistoryFavoritesButton() -> some View {
        Button(action: showHistoryFavoritesAction) {
            Label {
                Text("Bookmarks", bundle: .module, comment: "bookmarks button label")
            } icon: {
                Image("book", bundle: .module)
                    .font(.system(size: toolbarIconSize))
            }
        }
        .accessibilityIdentifier("button.history.favorites")
    }

    @ViewBuilder func tabsButton() -> some View {
        Menu {
            Button(action: { newTabAction() }) {
                Label {
                    Text("New Tab", bundle: .module, comment: "new tab button label")
                } icon: {
                    Image("plus", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.newTab")

            Button(action: duplicateTabAction) {
                Label {
                    Text("Duplicate Tab", bundle: .module, comment: "menu label that opens a new tab pointing at the same URL as the current tab")
                } icon: {
                    Image("plus.square.on.square", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.duplicateTab")
            .disabled(currentURL == nil)

            Button(action: closeTabAction) {
                Label {
                    Text("Close Tab", bundle: .module, comment: "close tab button label")
                } icon: {
                    Image("xmark", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.closeTab")

            Button(action: reopenClosedTabAction) {
                Label {
                    Text("Reopen Closed Tab", bundle: .module, comment: "menu label to restore the most recently closed tab")
                } icon: {
                    Image("arrow.clockwise", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.reopenClosedTab")
            .disabled(recentlyClosedTabURLs.isEmpty)

            Button(action: reloadAllTabsAction) {
                Label {
                    Text("Reload All Tabs", bundle: .module, comment: "menu label that triggers a reload on every open tab at once")
                } icon: {
                    Image("arrow.clockwise", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.reloadAllTabs")
            .disabled(tabs.isEmpty)

            Button(role: .destructive, action: closeOtherTabsAction) {
                Label {
                    Text("Close Other Tabs", bundle: .module, comment: "menu label to close every open tab except the currently selected one")
                } icon: {
                    Image("delete_sweep", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.closeOtherTabs")
            .disabled(tabs.count <= 1)

            Button(role: .destructive, action: closeAllTabsAction) {
                Label {
                    Text("Close All Tabs", bundle: .module, comment: "menu label to close every open tab")
                } icon: {
                    Image("delete_sweep", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.closeAllTabs")
            .disabled(tabs.count <= 1)
        } label: {
            Label {
                Text("Tabs", bundle: .module, comment: "tabs button label")
            } icon: {
                tabCountIcon()
                    .font(.system(size: toolbarIconSize))
            }
        } primaryAction: {
            tabListAction()
        }
        .accessibilityIdentifier("button.tabs")
    }

    @ViewBuilder func findBar() -> some View {
        HStack(spacing: 8) {
            TextField("Find on page", text: $findText)
                .textFieldStyle(.plain)
                .font(.system(size: 15))
                #if !SKIP
                .autocorrectionDisabled(true)
                #endif
                .onSubmit {
                    executeFindOnPage(findText)
                }
                .padding(.horizontal, 8)
                .padding(.vertical, 6)
                .background(Color.gray.opacity(0.15))
                .cornerRadius(8)
                .accessibilityIdentifier("field.findOnPage")

            Button(action: {
                executeFindOnPage(findText, backwards: true)
            }) {
                Image("chevron.left", bundle: .module)
                    .foregroundStyle(Color.accentColor)
            }
            .buttonStyle(.plain)
            .disabled(findText.isEmpty)
            .accessibilityIdentifier("button.findOnPage.previous")
            .accessibilityLabel(Text("Previous match", bundle: .module, comment: "accessibility label for the find-on-page Previous-match button"))

            Button(action: {
                executeFindOnPage(findText, backwards: false)
            }) {
                Image("chevron.right", bundle: .module)
                    .foregroundStyle(Color.accentColor)
            }
            .buttonStyle(.plain)
            .disabled(findText.isEmpty)
            .accessibilityIdentifier("button.findOnPage.next")
            .accessibilityLabel(Text("Next match", bundle: .module, comment: "accessibility label for the find-on-page Next-match button"))

            Button(action: {
                clearFindHighlights()
                showFindBar = false
                findText = ""
            }) {
                Image("xmark", bundle: .module)
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("button.findOnPage.close")
            .accessibilityLabel(Text("Close find on page", bundle: .module, comment: "accessibility label for the find-on-page close button"))
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color(white: 0.95))
    }

    @ViewBuilder func pageZoomBar() -> some View {
        HStack(spacing: 0) {
            Spacer()

            HStack(spacing: 2) {
                // Decrease zoom button (small A)
                Button(action: {
                    hapticFeedback()
                    settings.textZoom = max(settings.textZoom - 0.15, 0.5)
                }) {
                    Text(verbatim: "A")
                        .font(.system(size: 13, weight: .medium))
                        .frame(width: 40, height: 36)
                }
                .buttonStyle(.plain)
                .disabled(settings.textZoom <= 0.5)
                .accessibilityIdentifier("button.zoom.decrease")
                .accessibilityLabel(Text("Decrease text size", bundle: .module, comment: "accessibility label for the page-zoom decrease button"))

                // Current zoom percentage (tap to reset to 100%)
                Button(action: {
                    hapticFeedback()
                    settings.textZoom = 1.0
                }) {
                    let pct = Int((settings.textZoom * 100).rounded())
                    Text(verbatim: "\(pct)%")
                        .font(.system(size: 14, weight: .semibold))
                        .frame(width: 56, height: 36)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier("button.zoom.reset")
                .accessibilityLabel(Text("Reset text size", bundle: .module, comment: "accessibility label for the page-zoom reset button (tap to return to 100%)"))

                // Increase zoom button (large A)
                Button(action: {
                    hapticFeedback()
                    settings.textZoom = min(settings.textZoom + 0.15, 3.0)
                }) {
                    Text(verbatim: "A")
                        .font(.system(size: 19, weight: .medium))
                        .frame(width: 40, height: 36)
                }
                .buttonStyle(.plain)
                .disabled(settings.textZoom >= 3.0)
                .accessibilityIdentifier("button.zoom.increase")
                .accessibilityLabel(Text("Increase text size", bundle: .module, comment: "accessibility label for the page-zoom increase button"))
            }
            .background(Color.gray.opacity(0.15))
            .cornerRadius(8)

            Spacer()

            // Dismiss button
            Button(action: {
                showPageZoom = false
            }) {
                Image("xmark.circle.fill", bundle: .module)
                    .font(.system(size: 22))
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .padding(.leading, 8)
            .accessibilityIdentifier("button.zoom.dismiss")
            .accessibilityLabel(Text("Close page zoom", bundle: .module, comment: "accessibility label for the page-zoom dismiss button"))
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color(white: 0.95))
    }

    @ViewBuilder func tabCountIcon() -> some View {
        #if !SKIP
        // iOS: Label icons in toolbars require a static Image, not a dynamic view.
        // Pre-render the tab count badge to a UIImage via ImageRenderer and cache it.
        let img = Self.renderedTabCountImage(for: tabs.count)
        Image(uiImage: img)
            .renderingMode(.template)
        #else
        // Android/Skip: dynamic views work fine as toolbar icons
        ZStack {
            RoundedRectangle(cornerRadius: 5)
                .strokeBorder(lineWidth: 2.0)
                .frame(width: 28, height: 28)
            Text("\(tabs.count)")
                .font(.system(size: 14, weight: .bold))
        }
        #endif
    }

    #if !SKIP
    /// Cache of pre-rendered tab count badge images keyed by count.
    private static var tabCountImageCache: [Int: UIImage] = [:]

    /// Returns a cached UIImage for the given tab count, rendering it if needed.
    private static func renderedTabCountImage(for count: Int) -> UIImage {
        if let cached = tabCountImageCache[count] {
            return cached
        }
        let badge = ZStack {
            RoundedRectangle(cornerRadius: 5)
                .strokeBorder(lineWidth: 2.0)
                .frame(width: 28, height: 28)
            Text("\(count)")
                .font(.system(size: 14, weight: .bold))
        }
        .foregroundStyle(Color.primary)

        let renderer = ImageRenderer(content: badge)
        renderer.scale = UIScreen.main.scale
        let image = renderer.uiImage ?? UIImage()
        tabCountImageCache[count] = image
        return image
    }
    #endif

    @ViewBuilder func ellipsisMenu() -> some View {
        Menu {
            Button(action: reloadAction) {
                Label {
                    Text("Reload", bundle: .module, comment: "reload button label")
                } icon: {
                    Image("arrow.clockwise", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.reload")

            Button(action: homeAction) {
                Label {
                    Text("Home", bundle: .module, comment: "home button label")
                } icon: {
                    Image("house", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.home")

            Button(action: findOnPageAction) {
                Label {
                    Text("Find on Page", bundle: .module, comment: "find on page button label")
                } icon: {
                    Image("magnifyingglass", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.findOnPage")

            Divider()

            Button(action: favoriteAction) {
                Label {
                    if isCurrentPageFavorited {
                        Text("Remove from Favorites", bundle: .module, comment: "menu label when the current page is already saved as a favorite — tapping removes it")
                    } else {
                        Text("Add to Favorites", bundle: .module, comment: "add to favorites button label")
                    }
                } icon: {
                    Image("star", bundle: .module)
                }
            }
            .accessibilityIdentifier(isCurrentPageFavorited ? "menu.removeFavorite" : "menu.addFavorite")
            .disabled(currentURL == nil)

            ShareLink(item: currentState?.pageURL ?? fallbackURL) {
                Label {
                    Text("Share", bundle: .module, comment: "share button label")
                } icon: {
                    Image("ios_share", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.share")

            Button(action: copyURLAction) {
                Label {
                    Text("Copy URL", bundle: .module, comment: "menu label for copying the current page URL to the clipboard")
                } icon: {
                    Image("content_copy", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.copyURL")
            .disabled(currentState?.pageURL == nil)

            Button(action: pasteAndGoAction) {
                Label {
                    Text("Paste and Go", bundle: .module, comment: "menu label for pasting the clipboard contents into the URL bar and navigating to that page")
                } icon: {
                    Image("content_paste", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.pasteAndGo")

            Divider()

            Button(action: favoritesAction) {
                Label {
                    Text("Bookmarks", bundle: .module, comment: "bookmarks menu label")
                } icon: {
                    Image("book", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.bookmarks")

            Button(action: historyAction) {
                Label {
                    Text("History", bundle: .module, comment: "history menu label")
                } icon: {
                    Image("history", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.history")

            Button(action: downloadsAction) {
                Label {
                    Text("Downloads", bundle: .module, comment: "downloads menu label")
                } icon: {
                    Image("download", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.downloads")

            Divider()

            Button(action: pageZoomAction) {
                Label {
                    Text("Page Zoom", bundle: .module, comment: "page zoom button label")
                } icon: {
                    Image("textformat.size", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.pageZoom")

            Button(action: toggleDesktopSiteAction) {
                Label {
                    Text(settings.requestDesktopSite ? "Mobile Site" : "Desktop Site", bundle: .module, comment: "desktop/mobile site toggle")
                } icon: {
                    Image(settings.requestDesktopSite ? "smartphone" : "computer", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.desktopSite")

            Divider()

            Button(action: settingsAction) {
                Label {
                    Text("Settings", bundle: .module, comment: "settings button label")
                } icon: {
                    Image("gearshape", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.settings")
        } label: {
            Label {
                Text("Menu", bundle: .module, comment: "hamburger menu label")
            } icon: {
                Image("ellipsis", bundle: .module)
                    .font(.system(size: toolbarIconSize))
            }
        }
        .accessibilityIdentifier("button.menu")
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
        .accessibilityIdentifier("menu.historyItem.\(item.url)")
    }

    @MainActor func submitURL(text: String) {
        logger.log("URLBar submit")
        if let parsedURL = fieldToURL(text),
            ["http", "https", "file", "ftp", "netskip"].contains(parsedURL.scheme ?? "") {
            // HTTPS upgrade — when the user typed/pasted a bare `http://`
            // URL and the setting is on, rewrite it to `https://` before
            // dispatching. Localhost and IP literals are left alone since
            // those are typically explicitly chosen by developers.
            let finalURL: URL = Self.maybeUpgradeToHTTPS(parsedURL, enabled: settings.upgradeToHTTPS)
            logger.log("loading url: \(finalURL)")
            self.currentNavigator?.load(url: finalURL)
        } else {
            logger.log("URL search bar entry: \(text)")
            if let searchEngine = SearchEngine.lookup(id: settings.searchEngine),
               let queryURL = searchEngine.queryURL(text, Locale.current.identifier) {
                logger.log("search engine query URL: \(queryURL)")
                if let url = URL(string: queryURL) {
                    self.currentNavigator?.load(url: url)
                }
            }
        }
    }

    /// The home page URL — the user's custom `customHomeURL` setting if
    /// set (accepting either a fully-qualified URL or a bare host that
    /// we prepend `https://` to), otherwise the active search engine's
    /// home page.
    var homeURL: URL? {
        let trimmed = settings.customHomeURL.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmed.isEmpty {
            if let parsed = URL(string: trimmed), parsed.scheme != nil {
                return parsed
            }
            if let prefixed = URL(string: "https://" + trimmed), prefixed.host != nil {
                return prefixed
            }
        }
        if let homePage = SearchEngine.lookup(id: settings.searchEngine)?.homeURL,
           let homePageURL = URL(string: homePage) {
            return homePageURL
        } else {
            return nil
        }
    }

    /// Returns the URL with its scheme rewritten from `http` to `https`
    /// when the setting is enabled and the host isn't a developer-style
    /// local target. Returns the input unchanged otherwise.
    static func maybeUpgradeToHTTPS(_ url: URL, enabled: Bool) -> URL {
        if !enabled { return url }
        if url.scheme != "http" { return url }
        guard let host = url.host, !isLocalOrIPHost(host) else { return url }
        let str = url.absoluteString
        if str.hasPrefix("http://") {
            let upgraded = "https://" + String(str.dropFirst("http://".count))
            if let result = URL(string: upgraded) {
                return result
            }
        }
        return url
    }

    /// Hosts that should *not* be auto-upgraded to HTTPS — the developer
    /// almost certainly meant to hit them in cleartext. Covers `localhost`
    /// (with any port), single-label hostnames (no dot), and IPv4 dotted
    /// quads where TLS cert validation typically fails.
    private static func isLocalOrIPHost(_ host: String) -> Bool {
        if host == "localhost" || host.hasSuffix(".localhost") {
            return true
        }
        if !host.contains(".") {
            return true
        }
        // Crude IPv4 detection: all components numeric.
        let parts = host.split(separator: ".")
        if parts.count == 4, parts.allSatisfy({ Int($0) != nil }) {
            return true
        }
        return false
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
        if settings.buttonHaptics {
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
        // Capture each closing tab's URL before we remove it so the user
        // can reopen via the "Reopen Closed Tab" menu item. Blank tabs and
        // about:blank don't go onto the stack — there's nothing to restore.
        for id in ids {
            if let tab = tabs.first(where: { $0.id == id }),
               let url = tab.state.pageURL,
               !url.isEmpty, url != "about:blank" {
                recentlyClosedTabURLs.insert(url, at: 0)
                if recentlyClosedTabURLs.count > Self.recentlyClosedTabsLimit {
                    recentlyClosedTabURLs.removeLast()
                }
            }
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

    func reopenClosedTabAction() {
        guard !recentlyClosedTabURLs.isEmpty else { return }
        let url = recentlyClosedTabURLs.removeFirst()
        logger.info("reopenClosedTabAction: \(url)")
        hapticFeedback()
        newTabAction(url: url)
    }

    func closeAllTabsAction() {
        logger.info("closeAllTabsAction")
        hapticFeedback()
        confirmCloseAllTabs = true
    }

    /// Opens a fresh tab pointing at the current tab's URL. Honours the
    /// "Open Links in Background" setting so power users who keep that
    /// flipped on don't lose their place when forking a tab.
    func duplicateTabAction() {
        logger.info("duplicateTabAction url=\(self.currentURL ?? "nil")")
        guard let url = self.currentURL else { return }
        hapticFeedback()
        newTabAction(url: url, inBackground: settings.openLinksInBackground)
    }

    /// Fires `navigator.reload()` on every open tab. Useful after the
    /// network reconnects, a sign-in state changes, or a remote-config
    /// updates.
    func reloadAllTabsAction() {
        logger.info("reloadAllTabsAction count=\(self.tabs.count)")
        hapticFeedback()
        for tab in self.tabs {
            tab.navigator.reload()
        }
    }

    /// Closes every tab except the currently selected one — the classic
    /// "tidy up" shortcut after research sessions leave a dozen background
    /// tabs behind. Closed URLs flow through `closeTabs`, so each one is
    /// pushed onto the recently-closed stack and can be reopened via the
    /// Tabs menu's "Reopen Closed Tab" item.
    func closeOtherTabsAction() {
        logger.info("closeOtherTabsAction count=\(self.tabs.count) selected=\(self.selectedTab)")
        hapticFeedback()
        let otherIDs = Set(self.tabs.compactMap { $0.id == self.selectedTab ? nil : $0.id })
        guard !otherIDs.isEmpty else { return }
        closeTabs(otherIDs)
    }

    func performCloseAllTabs() {
        logger.info("performCloseAllTabs count=\(self.tabs.count)")
        let allIDs = Set(self.tabs.map(\.id))
        closeTabs(allIDs)
    }

    func findOnPageAction() {
        logger.info("findOnPageAction")
        hapticFeedback()
        #if !SKIP
        if let interaction = currentWebView?.findInteraction {
            interaction.presentFindNavigator(showingReplace: false)
            return
        }
        #endif
        // Fallback: show the custom find bar (used on Android, or iOS without findInteraction)
        showPageZoom = false
        showFindBar = true
        findText = ""
    }

    func executeFindOnPage(_ text: String) {
        executeFindOnPage(text, backwards: false)
    }

    /// Advances the in-page selection to the next or previous match of
    /// `text`. The `window.find` JS API takes
    /// `(text, caseSensitive, backwards, wrapAround)` — passing
    /// `backwards: true` walks matches in reverse so consecutive taps step
    /// backward through the page, with wrap-around so the buttons keep
    /// working at either end.
    func executeFindOnPage(_ text: String, backwards: Bool) {
        guard !text.isEmpty else { return }
        if let engine = currentViewModel?.navigator.webEngine {
            let backwardsArg = backwards ? "true" : "false"
            Task {
                _ = try? await engine.evaluate(js: "window.find('\(text.replacingOccurrences(of: "'", with: "\\'"))', false, \(backwardsArg), true)")
            }
        }
    }

    func clearFindHighlights() {
        if let engine = currentViewModel?.navigator.webEngine {
            Task {
                _ = try? await engine.evaluate(js: "window.getSelection().removeAllRanges()")
            }
        }
    }

    func newTabAction(url: String? = nil, inBackground: Bool = false) {
        logger.info("newTabAction url=\(url ?? "nil") inBackground=\(inBackground)")
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
        // Stay on the current tab when explicitly opening in the
        // background. Blank tabs always foreground because the user just
        // asked for "a new tab" and expects to land on it.
        if !inBackground || url == nil {
            self.selectedTab = vm.id
        }
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
        // Each opening of the tab overview starts with no filter applied.
        // Leaving stale search text would hide most tabs when the user
        // re-enters the sheet expecting to see everything.
        self.tabSearchText = ""
        self.showActiveTabs = true
    }

    func favoriteAction() {
        logger.info("favoriteAction isCurrentPageFavorited=\(isCurrentPageFavorited)")
        hapticFeedback()
        guard let url = self.currentURL else { return }
        if isCurrentPageFavorited {
            // Find the existing favorite row by URL and delete it.
            if let favorites = trying(operation: { try store.loadItems(type: .favorite, ids: []) }),
               let match = favorites.first(where: { $0.url == url }) {
                logger.info("removePageFromFavorite: \(url)")
                trying { try store.removeItems(type: .favorite, ids: [match.id]) }
            }
        } else {
            logger.info("addPageToFavorite: \(url)")
            trying {
                _ = try store.saveItems(type: .favorite, items: [PageInfo(url: url, title: currentState?.pageTitle ?? currentWebView?.title)])
            }
        }
        refreshFavoritedStatus()
    }

    /// Recompute whether the current page is in the favorites store.
    /// Called when the URL changes, when the menu opens, and right after
    /// the user toggles via `favoriteAction()` so the menu label flips
    /// without waiting for the next render trigger.
    func refreshFavoritedStatus() {
        guard let url = self.currentURL else {
            isCurrentPageFavorited = false
            return
        }
        let favorites = trying(operation: { try store.loadItems(type: .favorite, ids: []) }) ?? []
        isCurrentPageFavorited = favorites.contains(where: { $0.url == url })
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

    func downloadsAction() {
        logger.info("downloadsAction")
        hapticFeedback()
        showDownloads = true
    }

    /// Clears every tab's web cache (disk + memory + offline application
    /// cache) but explicitly leaves cookies and local storage intact so
    /// the user remains signed in everywhere they were. iOS / Android
    /// share their data stores across `WKWebView` / `WebView` instances,
    /// so dispatching the removal on each tab's engine guarantees the
    /// platform store is touched at least once.
    func clearWebCacheAction() {
        logger.info("clearWebCacheAction")
        hapticFeedback()
        let cacheTypes: Set<WebSiteDataType> = [.diskCache, .memoryCache, .offlineWebApplicationCache]
        for tab in self.tabs {
            if let engine = tab.navigator.webEngine {
                Task {
                    do {
                        try await engine.removeData(ofTypes: cacheTypes, modifiedSince: .distantPast)
                    } catch {
                        logger.warning("clearWebCacheAction: failed to clear cache: \(error)")
                    }
                }
            }
        }
    }

    func copyURLAction() {
        guard let pageURL = currentState?.pageURL, !pageURL.isEmpty else { return }
        logger.info("copyURLAction: \(pageURL)")
        hapticFeedback()
        #if SKIP
        let ctx = ProcessInfo.processInfo.androidContext
        let cm = ctx.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
        cm.setPrimaryClip(android.content.ClipData.newPlainText("URL", pageURL))
        #else
        UIPasteboard.general.string = pageURL
        #endif
    }

    func pasteAndGoAction() {
        logger.info("pasteAndGoAction")
        hapticFeedback()
        let clipboardText: String?
        #if SKIP
        let ctx = ProcessInfo.processInfo.androidContext
        let cm = ctx.getSystemService(android.content.Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
        if let clip = cm.primaryClip, clip.itemCount > 0 {
            clipboardText = clip.getItemAt(0).coerceToText(ctx)?.toString()
        } else {
            clipboardText = nil
        }
        #else
        clipboardText = UIPasteboard.general.string
        #endif
        guard let text = clipboardText?.trimmingCharacters(in: .whitespacesAndNewlines),
              !text.isEmpty else {
            logger.info("pasteAndGoAction: clipboard empty")
            return
        }
        // Reuse the URL bar's submit pipeline so the same heuristic (URL vs
        // search query) applies as if the user had typed it.
        submitURL(text: text)
    }

    func pageZoomAction() {
        logger.info("pageZoomAction")
        hapticFeedback()
        showFindBar = false
        showPageZoom = true
    }

    func toggleDesktopSiteAction() {
        logger.info("toggleDesktopSiteAction: \(!settings.requestDesktopSite)")
        hapticFeedback()
        settings.requestDesktopSite.toggle()
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
    // SKIP INSERT: @Suppress("MUST_BE_INITIALIZED_OR_FINAL_OR_ABSTRACT") // var to workaround Kotlin to 2 error: "Property must be initialized, be final, or be abstract."
    public let id: PageInfo.ID
    let navigator: WebViewNavigator
    let configuration: WebEngineConfiguration
    let store: WebBrowserStore
    var state = WebViewState()
    var urlTextField: String = ""

    /// Saved metadata from the database, used as fallback before the page loads
    var savedTitle: String = ""
    var savedURL: String = ""

    public init(id: PageInfo.ID, navigator: WebViewNavigator, configuration: WebEngineConfiguration, store: WebBrowserStore) {
        self.id = id
        self.navigator = navigator
        self.configuration = configuration
        self.store = store
    }
}

struct MiniAppLaunchItem: Identifiable {
    let appID: String
    var id: String { appID }
}

#endif

