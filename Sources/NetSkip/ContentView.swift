// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
#if SKIP || os(iOS)
import SkipWeb
#endif
import NetSkipModel

public struct ContentView: View {
    #if SKIP || os(iOS)
    let config: WebEngineConfiguration = {
        var rulePaths: [String] = []
        if let adsPath = Bundle.module.url(forResource: "block-ads", withExtension: "json")?.path {
            rulePaths.append(adsPath)
        }
        if let cookiesPath = Bundle.module.url(forResource: "block-cookies", withExtension: "json")?.path {
            rulePaths.append(cookiesPath)
        }
        return WebEngineConfiguration(
            javaScriptEnabled: true,
            contentBlockers: WebContentBlockerConfiguration(
                iOSRuleListPaths: rulePaths,
                androidMode: .custom(NetSkipAdBlockProvider())
            )
        )
    }()
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
    }
}

#if SKIP || os(iOS)
/// Cross-platform ad-blocking provider for Android.
/// Blocks requests to known ad/tracking domains by URL pattern matching.
final class NetSkipAdBlockProvider: AndroidContentBlockingProvider {
    /// Common ad/tracker domain substrings to block.
    private let blockedPatterns: [String] = [
        "doubleclick.net",
        "googlesyndication.com",
        "googleadservices.com",
        "google-analytics.com",
        "googletagmanager.com",
        "facebook.com/tr",
        "facebook.net/en_US/fbevents",
        "connect.facebook.net",
        "analytics.",
        "adservice.",
        "pagead2.googlesyndication",
        "ads.pubmatic.com",
        "cdn.taboola.com",
        "cdn.outbrain.com",
        "amazon-adsystem.com",
        "adnxs.com",
        "adsrvr.org",
        "criteo.com",
        "rubiconproject.com",
        "moatads.com",
        "scorecardresearch.com",
        "quantserve.com",
        "bluekai.com",
        "exelator.com",
        "turn.com",
        "serving-sys.com",
        "adhigh.net",
        "admob.com",
        "adcolony.com",
        "chartbeat.com",
        "hotjar.com",
    ]

    var persistentCosmeticRules: [AndroidCosmeticRule] { [] }

    func requestDecision(for request: AndroidBlockableRequest) -> AndroidRequestBlockDecision {
        // Don't block main frame navigations
        if request.isForMainFrame {
            return .allow
        }
        let urlString = request.url.absoluteString
        for pattern in blockedPatterns {
            if urlString.contains(pattern) {
                return .block
            }
        }
        return .allow
    }

    func navigationCosmeticRules(for page: AndroidPageContext) -> [AndroidCosmeticRule] {
        return []
    }
}
#endif

