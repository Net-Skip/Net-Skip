// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
#if SKIP || os(iOS)
import SkipWeb
#endif
import NetSkipModel
import AppFairUI

struct SettingsView : View {
    #if SKIP || os(iOS)
    var configuration: WebEngineConfiguration
    #endif
    var store: WebBrowserStore?
    var onClearCache: (() -> Void)? = nil

    @Environment(NetSkipSettings.self) var settings

    @State var confirmClearHistory: Bool = false
    @State var confirmClearFavorites: Bool = false
    @State var confirmClearCache: Bool = false
    @State var confirmClearAll: Bool = false

    @Environment(\.dismiss) var dismiss

    var body: some View {
        // `@Bindable` projects two-way bindings off the @Observable settings
        // instance so individual Toggle/Picker/TextField controls can write
        // back through `$settings.x`. Must be declared and used inline in
        // `body`; passing a plain `NetSkipSettings` parameter to a helper
        // would lose the projected-value subscript.
        @Bindable var settings = settings
        return AppFairSettings(bundle: .module) {
            Section {
                Picker(selection: $settings.appearance) {
                    Text("System", bundle: .module, comment: "settings appearance system label").tag("")
                    Text("Light", bundle: .module, comment: "settings appearance system label").tag("light")
                    Text("Dark", bundle: .module, comment: "settings appearance system label").tag("dark")
                } label: {
                    Label {
                        Text("Appearance", bundle: .module, comment: "settings appearance picker label")
                    } icon: {
                        Image("palette", bundle: .module)
                    }
                }
                .accessibilityIdentifier("picker.appearance")

                Toggle(isOn: $settings.buttonHaptics, label: {
                    Label {
                        Text("Haptic Feedback", bundle: .module, comment: "settings toggle label for button haptic feedback")
                    } icon: {
                        Image("vibration", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.haptics")
                Toggle(isOn: $settings.pageLoadHaptics, label: {
                    Label {
                        Text("Page Load Haptics", bundle: .module, comment: "settings toggle label for page load haptic feedback")
                    } icon: {
                        Image("notifications_active", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.pageLoadHaptics")
            }

            Section {
                Picker(selection: $settings.searchEngine) {
                    ForEach(SearchEngine.defaultSearchEngines, id: \.id) { engine in
                        Text(verbatim: engine.name())
                            .tag(engine.id)
                    }
                } label: {
                    Label {
                        Text("Search Engine", bundle: .module, comment: "settings picker label for the default search engine")
                    } icon: {
                        Image("travel_explore", bundle: .module)
                    }
                }
                .accessibilityIdentifier("picker.searchEngine")

                Toggle(isOn: $settings.searchSuggestions, label: {
                    Label {
                        Text("Search Suggestions", bundle: .module, comment: "settings toggle label for previewing search suggestions")
                    } icon: {
                        Image("lightbulb", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.searchSuggestions")

                HStack {
                    Image("house", bundle: .module)
                        .foregroundStyle(.secondary)
                    TextField(text: $settings.customHomeURL, prompt: Text("Home Page URL", bundle: .module, comment: "placeholder text for the custom home-page-URL field")) {
                        Text("Home Page URL", bundle: .module, comment: "accessibility label for the custom home-page-URL field")
                    }
                    #if !SKIP
                    .autocorrectionDisabled(true)
                    .textInputAutocapitalization(.never)
                    .keyboardType(.URL)
                    #endif
                    .accessibilityIdentifier("field.customHomeURL")
                }
            }

            Section("Display") {
                Toggle(isOn: $settings.hideStatusBar, label: {
                    Label {
                        Text("Hide Status Bar", bundle: .module, comment: "settings toggle label for hiding the system status bar so web pages render edge-to-edge")
                    } icon: {
                        Image("fullscreen", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.hideStatusBar")
            }

            Section("Privacy") {
                Toggle(isOn: $settings.enableJavaScript, label: {
                    Label {
                        Text("Enable JavaScript", bundle: .module, comment: "settings toggle label for enabling JavaScript")
                    } icon: {
                        Image("code", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.javascript")
                Toggle(isOn: $settings.upgradeToHTTPS, label: {
                    Label {
                        Text("Upgrade to HTTPS", bundle: .module, comment: "settings toggle label for auto-upgrading plain HTTP requests to HTTPS")
                    } icon: {
                        Image("https", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.upgradeToHTTPS")
            }

            Section("Downloads") {
                Toggle(isOn: $settings.promptForDownloads, label: {
                    Label {
                        Text("Prompt for File Downloads", bundle: .module, comment: "settings toggle label for confirming each download before it starts")
                    } icon: {
                        Image("download", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.promptForDownloads")
            }

            Section("Tabs") {
                Toggle(isOn: $settings.openLinksInBackground, label: {
                    Label {
                        Text("Open Links in Background", bundle: .module, comment: "settings toggle label that keeps the current tab focused when opening links in new tabs")
                    } icon: {
                        Image("tab", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.openLinksInBackground")
            }

            Section {
                Toggle(isOn: $settings.blockAds, label: {
                    Label {
                        Text("Block Ads", bundle: .module, comment: "settings toggle label for blocking ads")
                    } icon: {
                        Image("shield", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.blockAds")
                Toggle(isOn: $settings.blockTrackers, label: {
                    Label {
                        Text("Block Trackers", bundle: .module, comment: "settings toggle label for blocking trackers")
                    } icon: {
                        Image("block", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.blockTrackers")
                Toggle(isOn: $settings.blockCookieBanners, label: {
                    Label {
                        Text("Block Cookie Banners", bundle: .module, comment: "settings toggle label for blocking cookie consent banners")
                    } icon: {
                        Image("cookie", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.blockCookieBanners")

                NavigationLink {
                    NetSkipDomainListEditor(
                        title: Text("Allowed Sites", bundle: .module, comment: "title for the whitelisted sites editor"),
                        descriptionText: Text("Sites listed here bypass content blocking. Use bare domains like example.com or wildcards like *.example.com.", bundle: .module, comment: "description for whitelisted sites editor"),
                        prompt: Text("example.com", bundle: .module, comment: "placeholder for entering a whitelisted domain"),
                        emptyMessage: Text("No allowed sites yet.", bundle: .module, comment: "empty state for the whitelisted sites editor"),
                        rawText: $settings.contentBlockingWhitelistedDomains
                    )
                } label: {
                    Label {
                        Text("Allowed Sites", bundle: .module, comment: "settings row label for the whitelisted sites editor")
                    } icon: {
                        Image("public", bundle: .module)
                    }
                }
                .accessibilityIdentifier("link.allowedSites")

                NavigationLink {
                    NetSkipDomainListEditor(
                        title: Text("Custom Block Rules", bundle: .module, comment: "title for the custom blocked patterns editor"),
                        descriptionText: Text("Block any request whose URL contains one of these substrings. Example: tracker.example.com or /analytics/.", bundle: .module, comment: "description for custom blocked patterns editor"),
                        prompt: Text("tracker.example.com", bundle: .module, comment: "placeholder for entering a custom blocked pattern"),
                        emptyMessage: Text("No custom rules yet.", bundle: .module, comment: "empty state for the custom blocked patterns editor"),
                        rawText: $settings.contentBlockingCustomBlockedPatterns
                    )
                } label: {
                    Label {
                        Text("Custom Block Rules", bundle: .module, comment: "settings row label for the custom blocked patterns editor")
                    } icon: {
                        Image("rule", bundle: .module)
                    }
                }
                .accessibilityIdentifier("link.customBlockRules")
            } header: {
                Text("Content Blocking", bundle: .module, comment: "settings header for content blocking customization")
            } footer: {
                Text("Changes to content blocking apply on the next page load.", bundle: .module, comment: "settings footer explaining when content blocking changes take effect")
            }

            Section("Experimental") {
                Toggle(isOn: $settings.enableMiniApps, label: {
                    Label {
                        Text("MiniApps", bundle: .module, comment: "settings toggle label for enabling miniapps experimental feature")
                    } icon: {
                        Image("widgets", bundle: .module)
                    }
                })
                .accessibilityIdentifier("toggle.miniApps")
            }

            Section("Data") {
                Button(role: .destructive) {
                    confirmClearHistory = true
                } label: {
                    Label {
                        Text("Clear History", bundle: .module, comment: "settings button to clear browsing history")
                    } icon: {
                        Image("history", bundle: .module)
                    }
                }
                .accessibilityIdentifier("button.clearHistory")
                .confirmationDialog("Clear all browsing history?", isPresented: $confirmClearHistory) {
                    Button("Clear History", role: .destructive) {
                        trying { try store?.removeItems(type: .history, ids: []) }
                    }
                }

                Button(role: .destructive) {
                    confirmClearFavorites = true
                } label: {
                    Label {
                        Text("Clear Favorites", bundle: .module, comment: "settings button to clear favorites")
                    } icon: {
                        Image("star", bundle: .module)
                    }
                }
                .accessibilityIdentifier("button.clearFavorites")
                .confirmationDialog("Clear all favorites?", isPresented: $confirmClearFavorites) {
                    Button("Clear Favorites", role: .destructive) {
                        trying { try store?.removeItems(type: .favorite, ids: []) }
                    }
                }

                Button(role: .destructive) {
                    confirmClearCache = true
                } label: {
                    Label {
                        Text("Clear Cache", bundle: .module, comment: "settings button to clear the web cache without touching cookies or saved bookmarks")
                    } icon: {
                        Image("cached", bundle: .module)
                    }
                }
                .accessibilityIdentifier("button.clearCache")
                .confirmationDialog(
                    Text("Clear web cache?", bundle: .module, comment: "title of the clear-cache confirmation dialog"),
                    isPresented: $confirmClearCache,
                    titleVisibility: .visible
                ) {
                    Button(role: .destructive) {
                        onClearCache?()
                    } label: {
                        Text("Clear Cache", bundle: .module, comment: "destructive confirm button in the Clear Cache dialog")
                    }
                    .accessibilityIdentifier("button.clearCache.confirm")
                }

                Button(role: .destructive) {
                    confirmClearAll = true
                } label: {
                    Label {
                        Text("Clear All Browsing Data", bundle: .module, comment: "settings button to clear all data")
                    } icon: {
                        Image("delete_forever", bundle: .module)
                    }
                }
                .accessibilityIdentifier("button.clearAllData")
                .confirmationDialog("Clear all browsing data including history, favorites, and open tabs?", isPresented: $confirmClearAll) {
                    Button("Clear All Data", role: .destructive) {
                        trying {
                            try store?.removeItems(type: .history, ids: [])
                            try store?.removeItems(type: .favorite, ids: [])
                        }
                    }
                }
            }

            Section {
                // FIXME: should not need to explicitly specify Base.lproj; it should load as a fallback language automatically
                let aboutURL = Bundle.module.url(forResource: "about", withExtension: "html")
                if let aboutPage = aboutURL {
                    #if !SKIP
                    // FIXME: need Skip support for localizedInfoDictionary / infoDictionary
                    let dict = Bundle.main.localizedInfoDictionary ?? Bundle.main.infoDictionary
                    let appName = dict?["CFBundleDisplayName"] as? String ?? "App"
                    let appVersion = dict?["CFBundleShortVersionString"] as? String ?? "0.0.0"
                    #else
                    let appName = "App"
                    let appVersion = "0.0.0"
                    #endif
                    NavigationLink {
                        // Cannot local local resource paths on Android
                        // https://github.com/skiptools/skip-web/issues/1
                        //WebView(url: aboutPage)
                        #if !os(macOS)
                        VStack(spacing: 0.0) {
                            TitleView()
                            let aboutHTML = ((try? String(contentsOf: aboutPage)) ?? "error loading local content")
                                .replacingOccurrences(of: "APP_VERSION", with: appVersion)
                            WebView(html: aboutHTML)
                        }
                        #endif
                    } label: {
                        Label {
                            Text("About \(appName) \(appVersion)", bundle: .module, comment: "settings title menu for about app in the form ”About APP_NAME APP_VERSION”")
                        } icon: {
                            Image("info", bundle: .module)
                        }
                    }
                    .accessibilityIdentifier("link.about")
                }
            }
        }
        .navigationTitle(Text("Settings", bundle: .module, comment: "settings sheet title"))
        #if !os(macOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    self.dismiss()
                } label: {
                    Text("Done", bundle: .module, comment: "done button title")
                        .bold()
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier("button.settings.done")
            }
        }
    }
}
