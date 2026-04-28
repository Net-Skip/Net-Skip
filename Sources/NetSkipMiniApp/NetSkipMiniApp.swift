// SPDX-License-Identifier: GPL-2.0-or-later
import Foundation
import OSLog
import SwiftUI
import SkipMiniApp
import SkipMiniAppModel

let miniAppLogger = Logger(subsystem: "NetSkip", category: "MiniApp")

/// A catalog entry for an embedded sample miniapp.
public struct MiniAppCatalogEntry: Identifiable {
    public let id: String
    public let name: String
    public let description: String
    public let directoryName: String

    public init(id: String, name: String, description: String, directoryName: String) {
        self.id = id
        self.name = name
        self.description = description
        self.directoryName = directoryName
    }
}

/// The built-in sample miniapps bundled with the app.
public let sampleMiniApps: [MiniAppCatalogEntry] = [
    MiniAppCatalogEntry(
        id: "showcase-demo",
        name: "Showcase Demo",
        description: "Counter, file storage, network, and i18n demo",
        directoryName: "showcase-demo.ma"
    ),
    MiniAppCatalogEntry(
        id: "tap-game",
        name: "Tap Game",
        description: "Fast-paced tapping game with scoring",
        directoryName: "tap-game.ma"
    ),
    MiniAppCatalogEntry(
        id: "tabbed-app",
        name: "Tabbed App",
        description: "Multi-tab app with navigation",
        directoryName: "tabbed-app.ma"
    ),
    MiniAppCatalogEntry(
        id: "weather-app",
        name: "Weather",
        description: "Live forecast with Open-Meteo",
        directoryName: "weather-app.ma"
    ),
]

/// Returns the URL to the miniapp directory in the bundle, if found.
public func miniAppURL(for entry: MiniAppCatalogEntry) -> URL? {
    return Bundle.module.url(forResource: entry.directoryName, withExtension: nil, subdirectory: "MiniApps")
}

/// The base directory for miniapp persistent storage.
public var miniAppStorageBaseDirectory: URL {
    let dir = URL.applicationSupportDirectory.appendingPathComponent("miniapp-storage")
    try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    return dir
}

#if os(iOS) || SKIP
/// A view that hosts a miniapp from a catalog entry.
public struct MiniAppHostingView: View {
    let entry: MiniAppCatalogEntry
    let onDismiss: (() -> Void)?

    public init(entry: MiniAppCatalogEntry, onDismiss: (() -> Void)? = nil) {
        self.entry = entry
        self.onDismiss = onDismiss
    }

    public var body: some View {
        if let url = miniAppURL(for: entry) {
            MiniAppHostView(
                directoryURL: url,
                namespace: "skip",
                modules: [
                    .fileSystem(baseDirectory: miniAppStorageBaseDirectory),
                    .network,
                    .i18n,
                    .logging(onLog: { logEntry in
                        miniAppLogger.info("[\(entry.id)] \(logEntry.message)")
                    })
                ],
                onDismiss: onDismiss
            )
        } else {
            VStack {
                Text("MiniApp not found: \(entry.directoryName)")
                    .foregroundStyle(.secondary)
            }
        }
    }
}
#endif
