// SPDX-License-Identifier: GPL-2.0-or-later
import SwiftUI
#if SKIP || os(iOS)
import SkipWeb
#endif
import NetSkipModel

public struct ContentView: View {
    #if SKIP || os(iOS)
    /// Shared Android blocker that reads category toggles and custom patterns
    /// from `UserDefaults` on every request, so settings changes take effect
    /// on the next resource load.
    static let adBlockProvider = NetSkipAdBlockProvider()

    let config: WebEngineConfiguration = {
        return WebEngineConfiguration(
            javaScriptEnabled: true,
            contentBlockers: netSkipMakeContentBlockerConfiguration(provider: ContentView.adBlockProvider)
        )
    }()
    #endif
    let store = try! NetSkipWebBrowserStore(url: URL.documentsDirectory.appendingPathComponent("netskip.sqlite"))

    @State private var settings = NetSkipSettings()

    public init() {
    }

    public var body: some View {
        #if SKIP || os(iOS)
        BrowserTabView(configuration: config, store: store)
            .environment(settings)
        #else
        Text("Net Skip requires iOS")
            .environment(settings)
        #endif
    }
}
