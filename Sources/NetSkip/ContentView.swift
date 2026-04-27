// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
#if SKIP || os(iOS)
import SkipWeb
#endif
import NetSkipModel

public struct ContentView: View {
    #if SKIP || os(iOS)
    let config = WebEngineConfiguration(javaScriptEnabled: true)
    #endif
    let store = try! NetSkipWebBrowserStore(url: URL.documentsDirectory.appendingPathComponent("netskip.sqlite"))

    public init() {
    }

    public var body: some View {
        NavigationStack {
            #if SKIP || os(iOS)
            BrowserTabView(configuration: config, store: store)
                #if SKIP
                .toolbar(.hidden, for: .navigationBar)
                #endif
            #else
            Text("Net Skip requires iOS")
            #endif
        }
        .task {
            await loadContentBlockerRules()
        }
    }

    @MainActor func loadContentBlockerRules() async {
        #if !SKIP && os(iOS)
        guard let ruleListStore = WebContentRuleListStore.default() else { return }

        for blockerID in [
            "block-ads",
            "block-cookies",
        ] {
            if let blockerURL = Bundle.module.url(forResource: blockerID, withExtension: "json") {
                logger.info("loading content blocker rules from: \(blockerURL)")
                do {
                    let blockerContents = try String(contentsOf: blockerURL)
                    let ruleList = try await ruleListStore.compileContentRuleList(forIdentifier: blockerID, encodedContentRuleList: blockerContents)
                    let ids = await ruleListStore.availableIdentifiers()
                    logger.info("loaded content blocker rule list: \(ruleList) ids=\(ids ?? [])")
                    NotificationCenter.default.post(name: .webContentRulesLoaded, object: blockerID)
                } catch {
                    logger.error("error loading content blocker rules from \(blockerURL): \(error)")
                }
            }
        }
        #endif
    }
}

