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
    @AppStorage("selectedTabState") var selectedTabState: String = "" // app storage does not support Int64 (PageInfo.ID), so we serialize it as a string

    public init(configuration: WebEngineConfiguration, store: WebBrowserStore) {
        self.configuration = configuration
        self.store = store
    }

    public var body: some View {
        browserTabView()
            .toolbar {
                ToolbarItemGroup(placement: toolbarPlacement) {
                    toolbarButton1()
                    Spacer()
                    toolbarButton2()
                    Spacer()
                    moreButton()
                    Spacer()
                    toolbarButton3()
                    Spacer()
                    toolbarButton4()
                }
            }
        .background(Color.clear) // Set the background color of the content to clear
        .toolbarBackground(Color.white.opacity(0.5), for: .bottomBar) // Set the translucent background color for the toolbar
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
                BrowserView(configuration: configuration, store: store, submitURL: { self.submitURL(text: $0) }, viewModel: tab, searchEngine: $searchEngine, searchSuggestions: $searchSuggestions, showSettings: $showSettings, showBottomBar: $showBottomBar, userAgent: $userAgent, blockAds: $blockAds, enableJavaScript: $enableJavaScript)
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
        .onChange(of: selectedTab) {
            self.showBottomBar = true // always show the bottom bar when we change tabs
            self.selectedTabState = selectedTab.description // persist the last selected tab so we can restore it when re-starting
        }
        .onAppear {
            logger.info("restoring active tabs")
            restoreActiveTabs()
            self.showBottomBar = true
        }
    }

    func settingsView() -> some View {
        SettingsView(configuration: configuration, appearance: $appearance, buttonHaptics: $buttonHaptics, pageLoadHaptics: $pageLoadHaptics, searchEngine: $searchEngine, searchSuggestions: $searchSuggestions, userAgent: $userAgent, blockAds: $blockAds, enableJavaScript: $enableJavaScript)
            #if !SKIP
            .environment(\.openURL, openURLAction(newTab: true))
            #endif
    }

    func restoreActiveTabs() {
        // restore the saved tabs
        let activeTabs = trying { try store.loadItems(type: PageInfo.PageType.active, ids: []) }

        for activeTab in activeTabs ?? [] {
            logger.log("restoring tab \(activeTab.id): \(activeTab.url ?? "NONE") title=\(activeTab.title ?? "")")
            let viewModel = newViewModel(activeTab)
            self.tabs.append(viewModel)
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

    func activeTabsView() -> some View {
        NavigationStack {
            PageInfoListView(type: PageInfo.PageType.active, store: store, onSelect: { pageInfo in
                logger.info("select tab: \(pageInfo.url ?? "NONE")")
                withAnimation {
                    self.selectedTab = pageInfo.id
                }
            }, onDelete: { pageInfos in
                //logger.info("delete tabs: \(pageInfos.map(\.url))")
                closeTabs(Set(pageInfos.map(\.id)))
            }, toolbarItems: {
                ToolbarItemGroup(placement: .bottomBar) {
                    Button(action: {
                        newTabAction()
                        showActiveTabs = false
                    }) {
                        Label {
                            Text("New Tab", bundle: .module, comment: "more button string for creating a new tab")
                        } icon: {
                            Image("plus", bundle: .module)
                        }
                    }
                    Spacer()
                    Button {
                        showActiveTabs = false
                    } label: {
                        Text("Done", bundle: .module, comment: "done button title")
                            .bold()
                    }
                }
            })
        }
        #if !SKIP
        .presentationDetents([.medium, .large])
        #endif
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

    var currentViewModel: BrowserViewModel! {
        #if SKIP
        // workaround for crash on Android because currentViewModel returns nil
        tabs.first(where: { $0.id == self.selectedTab })
            ?? newViewModel(PageInfo(url: nil))
        #else
        tabs.first(where: { $0.id == self.selectedTab })
        #endif
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

    func toolbarButton1() -> some View {
        backButton()
    }

    func toolbarButton2() -> some View {
        forwardButton()
    }

    func toolbarButton3() -> some View {
        showHistoryFavoritesButton()
    }

    func toolbarButton4() -> some View {
        tabListButton()
    }

    @ViewBuilder func showHistoryFavoritesButton() -> some View {
        Button(action: showHistoryFavoritesAction) {
            Label {
                Text("History and Favorites", bundle: .module, comment: "more button string for opening the history and favorites")
            } icon: {
                Image("book", bundle: .module)
            }
        }
        .accessibilityIdentifier("button.history.favorites")
    }

    @ViewBuilder func backButton() -> some View {
        let enabled = currentState?.canGoBack == true
        let backLabel = Label {
            Text("Back", bundle: .module, comment: "back button label")
        } icon: {
            Image("chevron.left", bundle: .module)
        }

        if isSkip || !enabled {
            // TODO: SkipUI does not support Menu with primaryAction in toolbar
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
            // TODO: SkipUI does not support Menu with primaryAction in toolbar
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

    @ViewBuilder func tabListButton() -> some View {
        Menu {
            tabListMenu()
        } label: {
            Label {
                Text("Tab List", bundle: .module, comment: "tab list action label")
            } icon: {
                Image("square.on.square", bundle: .module)
            }
        } primaryAction: {
            tabListAction()
        }
        .accessibilityIdentifier("button.tablist")
    }

    @ViewBuilder func newTabButton() -> some View {
        Menu {
            Button(action: newPrivateTabAction) {
                Label {
                    Text("New Private Tab", bundle: .module, comment: "more button string for creating a new private tab")
                } icon: {
                    Image("plus.square.fill.on.square.fill", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.button.newprivatetab")

            Button(action: { newTabAction() }) {
                Label {
                    Text("New Tab", bundle: .module, comment: "more button string for creating a new tab")
                } icon: {
                    Image("plus.square.on.square", bundle: .module)
                }
            }
            .accessibilityIdentifier("menu.button.newtab")
        } label: {
            Label {
                Text("New Tab", bundle: .module, comment: "new tab action label")
            } icon: {
                Image("plus.square.on.square", bundle: .module)
            }
        } primaryAction: {
            newTabAction()
        }
        .accessibilityIdentifier("button.newtab")
    }

    @ViewBuilder func newTabMenu() -> some View {
        // TODO
    }

    @ViewBuilder func tabListMenu() -> some View {
        // TODO
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
        logger.info("newTabAction")
        hapticFeedback()
        let info = PageInfo(url: url) // open to a blank page
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

    func moreButton() -> some View {
        Menu {
            Button(action: { newTabAction() }) {
                Label {
                    Text("New Tab", bundle: .module, comment: "more button string for creating a new tab")
                } icon: {
                    Image("plus.square.on.square", bundle: .module)
                }
            }
            .accessibilityIdentifier("button.new")

            Button(action: closeTabAction) {
                Label {
                    Text("Close Tab", bundle: .module, comment: "more button string for closing a tab")
                } icon: {
                    Image("xmark", bundle: .module)
                }
            }
            .accessibilityIdentifier("button.close")

            Divider()

            Button(action: reloadAction) {
                Label {
                    Text("Reload", bundle: .module, comment: "more button string for reloading the current page")
                } icon: {
                    Image("arrow.clockwise.circle", bundle: .module)
                }
            }
            .accessibilityIdentifier("button.reload")
            Button(action: homeAction) {
                Label(title: {
                    Text("Home", bundle: .module, comment: "home button label")
                }, icon: {
                    Image("house", bundle: .module)
                })
            }
            .accessibilityIdentifier("button.home")

            Divider()

            Button {
                logger.log("find on page button tapped")
                findOnPageAction()
            } label: {
                Label(title: {
                    Text("Find on Page", bundle: .module, comment: "more button string for finding on the current page")
                }, icon: {
                    Image("magnifyingglass", bundle: .module)
                })
            }

//            Button {
//                logger.log("text zoom button tapped")
//            } label: {
//                Text("Text Zoom", bundle: .module, comment: "more button string for text zoom")
//            }

//            Button {
//                logger.log("disable blocker button tapped")
//            } label: {
//                Text("Disable Blocker", bundle: .module, comment: "more button string for disabling the blocker")
//            }

            // share button
            ShareLink(item: currentState?.pageURL ?? fallbackURL)
                .disabled(currentState?.pageURL == nil)

            Button(action: favoriteAction) {
                Label {
                    Text("Favorite", bundle: .module, comment: "more button string for adding a favorite")
                } icon: {
                    // TODO: make star.filled when it is already a favorite
                    Image("star", bundle: .module)
                }
            }
            .accessibilityIdentifier("button.favorite")

            Divider()

            Button(action: favoritesAction) {
                Label {
                    Text("Favorites", bundle: .module, comment: "more button string for opening the favorites list")
                } icon: {
                    Image("list.star", bundle: .module)
                }
            }
            .accessibilityIdentifier("button.favorites")

            Button(action: historyAction) {
                Label {
                    Text("History", bundle: .module, comment: "more button string for opening the history")
                } icon: {
                    Image("calendar", bundle: .module)
                }
            }
            .accessibilityIdentifier("button.history")

            Button(action: settingsAction) {
                Label {
                    Text("Settings", bundle: .module, comment: "more button string for opening the settings")
                } icon: {
                    Image("gearshape", bundle: .module)
                }
            }
            .accessibilityIdentifier("button.settings")
        } label: {
            Label {
                Text("More", bundle: .module, comment: "more button label")
            } icon: {
                Image("ellipsis", bundle: .module)
            }
            .accessibilityIdentifier("button.more")
        }
    }
}

struct TitleView : View {
    @State var isBeating = false

    var body: some View {
        VStack(alignment: .center, spacing: 25.0) {
            VStack {
                Text("Net Skip", bundle: .module, comment: "title screen headline")
                    .font(Font.system(size: 35, weight: .bold))
                    .lineLimit(1)
                    .foregroundStyle(LinearGradient(colors: [.red, .blue, .green, .yellow], startPoint: .leading, endPoint: .trailing))
//                Text("(a humane web browser)", bundle: .module, comment: "title screen sub-headline")
//                    .font(Font.subheadline.bold())
            }
//            Image(systemName: "heart.fill")
//                .font(.largeTitle)
//                .foregroundStyle(.red)
//                .scaleEffect(isBeating ? 1.5 : 1.0)
//                .animation(.easeInOut(duration: 1).repeatForever(), value: isBeating)
//                .onAppear { isBeating = true }
        }
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

