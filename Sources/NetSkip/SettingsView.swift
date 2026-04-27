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

    @Binding var appearance: String
    @Binding var buttonHaptics: Bool
    @Binding var pageLoadHaptics: Bool
    @Binding var searchEngine: SearchEngine.ID
    @Binding var searchSuggestions: Bool
    @Binding var userAgent: String
    @Binding var enableJavaScript: Bool
    @Binding var enableMiniApps: Bool

    @State var confirmClearHistory: Bool = false
    @State var confirmClearFavorites: Bool = false
    @State var confirmClearAll: Bool = false

    @Environment(\.dismiss) var dismiss

    var body: some View {
        AppFairSettings(bundle: .module) {
            Section {
                Picker(selection: $appearance) {
                    Text("System", bundle: .module, comment: "settings appearance system label").tag("")
                    Text("Light", bundle: .module, comment: "settings appearance system label").tag("light")
                    Text("Dark", bundle: .module, comment: "settings appearance system label").tag("dark")
                } label: {
                    Text("Appearance", bundle: .module, comment: "settings appearance picker label")
                }

                Toggle(isOn: $buttonHaptics, label: {
                    Text("Haptic Feedback", bundle: .module, comment: "settings toggle label for button haptic feedback")
                })
                Toggle(isOn: $pageLoadHaptics, label: {
                    Text("Page Load Haptics", bundle: .module, comment: "settings toggle label for page load haptic feedback")
                })
            }

            Section {
                Picker(selection: $searchEngine) {
                    ForEach(SearchEngine.defaultSearchEngines, id: \.id) { engine in
                        Text(verbatim: engine.name())
                            .tag(engine.id)
                    }
                } label: {
                    Text("Search Engine", bundle: .module, comment: "settings picker label for the default search engine")
                }

                Toggle(isOn: $searchSuggestions, label: {
                    Text("Search Suggestions", bundle: .module, comment: "settings toggle label for previewing search suggestions")
                })
            }

            Section("Privacy") {
                Toggle(isOn: $enableJavaScript, label: {
                    Text("Enable JavaScript", bundle: .module, comment: "settings toggle label for enabling JavaScript")
                })
                HStack {
                    Text("Ad & Tracker Blocking", bundle: .module, comment: "settings label for content blocking status")
                    Spacer()
                    Text("On", bundle: .module, comment: "content blocking enabled label")
                        .foregroundStyle(.secondary)
                }
            }

            Section("Experimental") {
                Toggle(isOn: $enableMiniApps, label: {
                    Text("MiniApps", bundle: .module, comment: "settings toggle label for enabling miniapps experimental feature")
                })
            }

            Section("Data") {
                Button(role: .destructive) {
                    confirmClearHistory = true
                } label: {
                    Text("Clear History", bundle: .module, comment: "settings button to clear browsing history")
                }
                .confirmationDialog("Clear all browsing history?", isPresented: $confirmClearHistory) {
                    Button("Clear History", role: .destructive) {
                        trying { try store?.removeItems(type: .history, ids: []) }
                    }
                }

                Button(role: .destructive) {
                    confirmClearFavorites = true
                } label: {
                    Text("Clear Favorites", bundle: .module, comment: "settings button to clear favorites")
                }
                .confirmationDialog("Clear all favorites?", isPresented: $confirmClearFavorites) {
                    Button("Clear Favorites", role: .destructive) {
                        trying { try store?.removeItems(type: .favorite, ids: []) }
                    }
                }

                Button(role: .destructive) {
                    confirmClearAll = true
                } label: {
                    Text("Clear All Browsing Data", bundle: .module, comment: "settings button to clear all data")
                }
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
                        Text("About \(appName) \(appVersion)", bundle: .module, comment: "settings title menu for about app in the form ”About APP_NAME APP_VERSION”")
                    }
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
            }
        }
    }
}
