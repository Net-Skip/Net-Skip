// SPDX-License-Identifier: GPL-2.0-or-later
import Foundation
import SwiftUI

/// Centralised user-preferences store for Net-Skip.
///
/// Loads each value from `UserDefaults` once on init and writes it back via
/// the property's `didSet` whenever it changes. Lives as a single
/// `@Observable` singleton-style instance owned by `ContentView` and
/// injected through the SwiftUI environment so any descendant view can
/// read or mutate any setting without us threading bindings through
/// every container.
///
/// The keys mirror what was previously stored via `@AppStorage`, so
/// existing on-disk preferences survive the migration.
@MainActor
@Observable
public final class NetSkipSettings {
    private let defaults: UserDefaults

    // MARK: - Appearance

    public var appearance: String {
        didSet { defaults.set(appearance, forKey: "appearance") }
    }
    public var buttonHaptics: Bool {
        didSet { defaults.set(buttonHaptics, forKey: "buttonHaptics") }
    }
    public var pageLoadHaptics: Bool {
        didSet { defaults.set(pageLoadHaptics, forKey: "pageLoadHaptics") }
    }

    // MARK: - Search

    public var searchEngine: String {
        didSet { defaults.set(searchEngine, forKey: "searchEngine") }
    }
    public var searchSuggestions: Bool {
        didSet { defaults.set(searchSuggestions, forKey: "searchSuggestions") }
    }
    public var userAgent: String {
        didSet { defaults.set(userAgent, forKey: "userAgent") }
    }
    public var requestDesktopSite: Bool {
        didSet { defaults.set(requestDesktopSite, forKey: "requestDesktopSite") }
    }
    public var textZoom: Double {
        didSet { defaults.set(textZoom, forKey: "textZoom") }
    }

    // MARK: - Privacy / content blocking

    public var enableJavaScript: Bool {
        didSet { defaults.set(enableJavaScript, forKey: "enableJavaScript") }
    }
    public var blockAds: Bool {
        didSet { defaults.set(blockAds, forKey: "blockAds") }
    }
    public var blockTrackers: Bool {
        didSet { defaults.set(blockTrackers, forKey: "blockTrackers") }
    }
    public var blockCookieBanners: Bool {
        didSet { defaults.set(blockCookieBanners, forKey: "blockCookieBanners") }
    }
    public var contentBlockingWhitelistedDomains: String {
        didSet { defaults.set(contentBlockingWhitelistedDomains, forKey: "contentBlockingWhitelistedDomains") }
    }
    public var contentBlockingCustomBlockedPatterns: String {
        didSet { defaults.set(contentBlockingCustomBlockedPatterns, forKey: "contentBlockingCustomBlockedPatterns") }
    }

    // MARK: - Downloads

    public var promptForDownloads: Bool {
        didSet { defaults.set(promptForDownloads, forKey: "promptForDownloads") }
    }

    // MARK: - Experimental

    public var enableMiniApps: Bool {
        didSet { defaults.set(enableMiniApps, forKey: "enableMiniApps") }
    }

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        self.appearance = defaults.string(forKey: "appearance") ?? ""
        self.buttonHaptics = Self.boolValue(defaults, key: "buttonHaptics", defaultValue: true)
        self.pageLoadHaptics = Self.boolValue(defaults, key: "pageLoadHaptics", defaultValue: false)
        self.searchEngine = defaults.string(forKey: "searchEngine") ?? ""
        self.searchSuggestions = Self.boolValue(defaults, key: "searchSuggestions", defaultValue: true)
        self.userAgent = defaults.string(forKey: "userAgent") ?? ""
        self.requestDesktopSite = Self.boolValue(defaults, key: "requestDesktopSite", defaultValue: false)
        let storedZoom = defaults.double(forKey: "textZoom")
        self.textZoom = storedZoom == 0.0 ? 1.0 : storedZoom
        self.enableJavaScript = Self.boolValue(defaults, key: "enableJavaScript", defaultValue: true)
        self.blockAds = Self.boolValue(defaults, key: "blockAds", defaultValue: true)
        self.blockTrackers = Self.boolValue(defaults, key: "blockTrackers", defaultValue: true)
        self.blockCookieBanners = Self.boolValue(defaults, key: "blockCookieBanners", defaultValue: true)
        self.contentBlockingWhitelistedDomains = defaults.string(forKey: "contentBlockingWhitelistedDomains") ?? ""
        self.contentBlockingCustomBlockedPatterns = defaults.string(forKey: "contentBlockingCustomBlockedPatterns") ?? ""
        self.promptForDownloads = Self.boolValue(defaults, key: "promptForDownloads", defaultValue: true)
        self.enableMiniApps = Self.boolValue(defaults, key: "enableMiniApps", defaultValue: false)
    }

    /// `UserDefaults.bool(forKey:)` returns false for missing keys, which
    /// would silently flip toggles that default to true. Probe via
    /// `object(forKey:)` first so we can distinguish "unset" from "false".
    private static func boolValue(_ defaults: UserDefaults, key: String, defaultValue: Bool) -> Bool {
        if defaults.object(forKey: key) != nil {
            return defaults.bool(forKey: key)
        }
        return defaultValue
    }
}
