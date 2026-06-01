// SPDX-License-Identifier: GPL-2.0-or-later
//
// Self-contained download manager + SwiftUI list view for Net-Skip.
//
// This file is intentionally written so the types here could be lifted out
// into a future `skip-download` framework without depending on anything from
// Net-Skip's other modules. The only external coupling is:
//
//   * `SkipWeb.WebDownloadRequest` — the request descriptor produced by the
//     embedded `WebView` when WebKit / Android WebView decides a navigation
//     should be a download. Any future generic version would re-declare an
//     equivalent struct.
//   * SwiftUI / Foundation — available cross-platform via Skip.
//
// Idiomatic file destinations match the native browser conventions on each
// platform:
//
//   iOS (Safari):  ~/Documents/Downloads — visible in the Files app under
//                  this app, and accessible from a share sheet.
//   Android (Chrome): the public Downloads directory
//                  (`Environment.DIRECTORY_DOWNLOADS`), which is the same
//                  folder Chrome and other browsers write to.

import Foundation
import SwiftUI
import SkipWeb
#if !SKIP
import UIKit
#endif

#if SKIP
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Environment
import androidx.core.content.FileProvider
#endif

/// Posted on `NotificationCenter.default` the moment a new download is
/// enqueued. UI code (e.g. a tab view) can use this to auto-present the
/// downloads sheet without having to be passed any direct binding.
public extension Notification.Name {
    static let netSkipDownloadEnqueued = Notification.Name("netSkipDownloadEnqueued")
}

// MARK: - Model

/// The lifecycle of a single download.
public enum NetSkipDownloadState: Equatable, Sendable {
    case pending
    case downloading
    case completed
    case cancelled
    case failed(String)
}

/// A single in-flight or finished download. `Observable` so SwiftUI views
/// rebuild as `bytesWritten` / `state` change.
@MainActor
@Observable
public final class NetSkipDownloadItem: Identifiable {
    public let id: UUID = UUID()
    public let url: URL?
    public let filename: String
    public let mimeType: String?
    public let startedAt: Date = Date()

    public var localURL: URL?
    public var state: NetSkipDownloadState = .pending
    public var bytesWritten: Int64 = 0
    public var totalBytes: Int64 = 0
    public var lastProgressAt: Date = Date()

    /// Set by the manager when the URL task is created so `cancel()` can stop
    /// the in-flight download. Marked `nonisolated(unsafe)` because it's
    /// read from the delegate queue / write from MainActor.
    nonisolated(unsafe) fileprivate var sessionTask: URLSessionTask?

    init(url: URL?, filename: String, mimeType: String?, expectedSize: Int64?) {
        self.url = url
        self.filename = filename
        self.mimeType = mimeType
        if let expectedSize = expectedSize, expectedSize > 0 {
            self.totalBytes = expectedSize
        }
    }

    /// Fraction in [0,1] — returns 0 when the total length is unknown.
    public var progress: Double {
        guard totalBytes > 0 else { return 0 }
        return min(1.0, Double(bytesWritten) / Double(totalBytes))
    }

    /// Average throughput in bytes/second since the download started.
    public var bytesPerSecond: Double {
        let elapsed = lastProgressAt.timeIntervalSince(startedAt)
        guard elapsed > 0.0 else { return 0 }
        return Double(bytesWritten) / elapsed
    }

    /// Estimated time remaining in seconds. nil if the total is unknown
    /// or we don't have enough data yet to make a guess.
    public var estimatedRemainingSeconds: Double? {
        guard case .downloading = state else { return nil }
        guard totalBytes > 0, bytesWritten > 0 else { return nil }
        let rate = bytesPerSecond
        guard rate > 0 else { return nil }
        let remaining = Double(totalBytes - bytesWritten) / rate
        if remaining.isFinite && remaining >= 0 { return remaining }
        return nil
    }

    public var isRunning: Bool {
        if case .downloading = state { return true }
        if case .pending = state { return true }
        return false
    }
}

// MARK: - Manager

/// Owns the list of downloads and the on-disk destination directory.
/// `Observable` so the menu badge / list view rebuild when downloads appear.
@MainActor
@Observable
public final class NetSkipDownloadManager {
    public static let shared = NetSkipDownloadManager()

    public private(set) var downloads: [NetSkipDownloadItem] = []
    public let downloadsDirectory: URL

    public init(downloadsDirectory: URL? = nil) {
        self.downloadsDirectory = downloadsDirectory ?? Self.platformDownloadsDirectory()
        try? FileManager.default.createDirectory(
            at: self.downloadsDirectory,
            withIntermediateDirectories: true,
            attributes: nil
        )
        logger.log("downloads directory: \(self.downloadsDirectory.path)")
    }

    /// Idiomatic per-platform downloads location.
    private static func platformDownloadsDirectory() -> URL {
        #if SKIP
        // Android: the public Downloads directory (same as Chrome).
        let dir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
        return URL(fileURLWithPath: dir.getAbsolutePath())
        #else
        // iOS: Documents/Downloads — what Safari uses when saving via the
        // share sheet, and what surfaces in the Files app.
        let documents: URL
        if let docs = try? FileManager.default.url(
            for: .documentDirectory, in: .userDomainMask,
            appropriateFor: nil, create: true
        ) {
            documents = docs
        } else {
            documents = FileManager.default.temporaryDirectory
        }
        return documents.appendingPathComponent("Downloads", isDirectory: true)
        #endif
    }

    /// Begin a new download for the WebKit / WebView request.
    @discardableResult
    public func enqueue(_ request: WebDownloadRequest) -> NetSkipDownloadItem {
        let filename = Self.sanitizeFilename(
            request.suggestedFilename
                ?? request.url?.lastPathComponent
                ?? "download",
            mimeType: request.mimeType,
            url: request.url,
            contentDisposition: request.contentDisposition
        )
        let item = NetSkipDownloadItem(
            url: request.url,
            filename: filename,
            mimeType: request.mimeType,
            expectedSize: request.contentLength
        )
        downloads.insert(item, at: 0)
        NotificationCenter.default.post(name: .netSkipDownloadEnqueued, object: item)
        startTask(for: item)
        return item
    }

    public func cancel(_ item: NetSkipDownloadItem) {
        item.sessionTask?.cancel()
        if case .pending = item.state {
            item.state = .cancelled
        }
    }

    /// Creates the per-download `URLSession` + delegate and resumes the task.
    /// Using a `URLSessionDataDelegate` instead of `URLSession.bytes(from:)`
    /// matters on Skip — the async-bytes iterator reads one byte at a time
    /// through `InputStream.read()` per call, which made multi-megabyte
    /// downloads crawl on Android. The delegate path receives bytes in
    /// `didReceive data:` chunks straight from OkHttp on Android and from
    /// URLSession's native loader on iOS.
    private func startTask(for item: NetSkipDownloadItem) {
        guard let url = item.url else {
            item.state = .failed("Missing URL")
            return
        }
        let destination = uniqueDestination(for: item.filename)
        item.localURL = destination
        let delegate = NetSkipDownloadTaskDelegate(item: item, destinationPath: destination.path)
        let config = URLSessionConfiguration.ephemeral
        let session = URLSession(configuration: config, delegate: delegate, delegateQueue: nil)
        delegate.session = session
        let task = session.dataTask(with: URLRequest(url: url))
        item.sessionTask = task
        item.state = .downloading
        task.resume()
    }

    /// Removes a finished/cancelled item from the visible list.
    /// In-flight downloads are cancelled first.
    public func remove(_ item: NetSkipDownloadItem) {
        cancel(item)
        downloads.removeAll(where: { $0.id == item.id })
    }

    public func clearFinished() {
        downloads.removeAll(where: { !$0.isRunning })
    }

    // MARK: - URL delegate

    /// Receives chunked body data from the URLSession data task and writes it
    /// to disk. Lives one-per-download — we don't share a session across
    /// downloads so each task's `invalidateAndCancel` cleans up cleanly.
    final class NetSkipDownloadTaskDelegate: NSObject, URLSessionDataDelegate {
        let item: NetSkipDownloadItem
        let destinationPath: String
        var session: URLSession?
        private var writer: NetSkipDownloadWriter?

        init(item: NetSkipDownloadItem, destinationPath: String) {
            self.item = item
            self.destinationPath = destinationPath
        }

        func urlSession(
            _ session: URLSession,
            dataTask: URLSessionDataTask,
            didReceive response: URLResponse,
            completionHandler: @escaping (URLSession.ResponseDisposition) -> Void
        ) {
            // The HTTP response carries the authoritative content length —
            // override any pre-flight estimate from the WebDownloadRequest.
            let advertisedLength = response.expectedContentLength
            Task { @MainActor in
                if advertisedLength > 0 {
                    item.totalBytes = max(item.totalBytes, advertisedLength)
                }
            }
            do {
                writer = try NetSkipDownloadWriter(path: destinationPath)
                completionHandler(.allow)
            } catch {
                completionHandler(.cancel)
                finish(error: error)
            }
        }

        func urlSession(_ session: URLSession, dataTask: URLSessionDataTask, didReceive data: Data) {
            do {
                try writer?.append(data)
                let count = Int64(data.count)
                Task { @MainActor in
                    item.bytesWritten += count
                    item.lastProgressAt = Date()
                }
            } catch {
                dataTask.cancel()
                finish(error: error)
            }
        }

        func urlSession(_ session: URLSession, task: URLSessionTask, didCompleteWithError error: Error?) {
            writer?.close()
            writer = nil
            finish(error: error)
        }

        private var finished = false
        private func finish(error: Error?) {
            if finished { return }
            finished = true
            let path = destinationPath
            let cancelled: Bool
            if let nserr = error as NSError?,
               nserr.domain == NSURLErrorDomain, nserr.code == NSURLErrorCancelled {
                cancelled = true
            } else {
                cancelled = false
            }
            let resolvedError = error
            Task { @MainActor in
                if cancelled {
                    try? FileManager.default.removeItem(atPath: path)
                    item.localURL = nil
                    item.state = .cancelled
                } else if let resolvedError {
                    try? FileManager.default.removeItem(atPath: path)
                    item.localURL = nil
                    item.state = .failed(String(describing: resolvedError))
                } else {
                    if item.totalBytes <= 0 {
                        item.totalBytes = item.bytesWritten
                    }
                    item.state = .completed
                }
                item.sessionTask = nil
            }
            session?.finishTasksAndInvalidate()
        }
    }

    /// Small platform abstraction: iOS uses `FileHandle`, Skip writes through a
    /// `BufferedOutputStream` so the platform layer can flush in its own way.
    final class NetSkipDownloadWriter {
        #if SKIP
        private let out: java.io.BufferedOutputStream
        init(path: String) throws {
            let file = java.io.File(path)
            file.getParentFile()?.mkdirs()
            self.out = java.io.BufferedOutputStream(java.io.FileOutputStream(file))
        }
        func append(_ data: Data) throws {
            let bytes = data.platformValue
            out.write(bytes, 0, bytes.size)
        }
        func close() {
            do { out.close() } catch { logger.log("download writer close failed: \(error)") }
        }
        #else
        private let handle: FileHandle
        init(path: String) throws {
            FileManager.default.createFile(atPath: path, contents: nil, attributes: nil)
            self.handle = try FileHandle(forWritingTo: URL(fileURLWithPath: path))
        }
        func append(_ data: Data) throws {
            try handle.write(contentsOf: data)
        }
        func close() {
            try? handle.close()
        }
        #endif
    }

    // MARK: - Filename hygiene

    private static let extensionByMime: [String: String] = [
        "application/zip": "zip",
        "application/x-zip-compressed": "zip",
        "application/gzip": "gz",
        "application/x-tar": "tar",
        "application/json": "json",
        "application/pdf": "pdf",
        "application/octet-stream": "",
        "image/png": "png",
        "image/jpeg": "jpg",
        "image/gif": "gif",
        "image/svg+xml": "svg",
        "image/webp": "webp",
        "text/plain": "txt",
        "text/html": "html",
        "text/csv": "csv",
        "application/vnd.android.package-archive": "apk",
        "application/x-apple-diskimage": "dmg",
    ]

    /// What this download will be saved as on disk — applies all the same
    /// fixes (Android URLUtil's `.bin` substitution, Content-Disposition
    /// parsing, MIME → extension mapping) that `enqueue(_:)` would. Use this
    /// in any UI that wants to preview the filename *before* committing to
    /// the download, e.g. the "Download <name>?" confirmation prompt.
    public static func resolvedFilename(for request: WebDownloadRequest) -> String {
        return sanitizeFilename(
            request.suggestedFilename
                ?? request.url?.lastPathComponent
                ?? "download",
            mimeType: request.mimeType,
            url: request.url,
            contentDisposition: request.contentDisposition
        )
    }

    static func sanitizeFilename(_ raw: String, mimeType: String?, url: URL? = nil, contentDisposition: String? = nil) -> String {
        var name = raw
        // Strip path separators that a hostile filename header could contain.
        name = name.replacingOccurrences(of: "/", with: "_")
        name = name.replacingOccurrences(of: "\\", with: "_")
        if name.isEmpty || name == "." || name == ".." {
            name = "download"
        }
        // Android's `URLUtil.guessFileName` rewrites the extension to match
        // the response MIME type when they disagree, so `appindex.json`
        // served as `application/octet-stream` becomes `appindex.bin`.
        // Recover the original by parsing the `filename=` parameter from
        // the Content-Disposition header — that is what the server
        // explicitly intends the file to be called.
        let ext = URL(fileURLWithPath: name).pathExtension.lowercased()
        if ext == "bin" || ext.isEmpty {
            if let cdName = filenameFromContentDisposition(contentDisposition) {
                name = cdName
            } else if let urlLastComponent = url?.lastPathComponent {
                let urlExt = URL(fileURLWithPath: urlLastComponent).pathExtension.lowercased()
                if !urlExt.isEmpty, urlExt != "bin" {
                    name = urlLastComponent
                }
            }
        }
        // If the filename has no extension but the MIME type implies one,
        // append it so the Files app / Chrome opens with the right viewer.
        let finalExt = URL(fileURLWithPath: name).pathExtension
        if finalExt.isEmpty, let mime = mimeType?.lowercased(),
           let mapped = extensionByMime[mime], !mapped.isEmpty {
            name = "\(name).\(mapped)"
        }
        return name
    }

    /// Extracts the `filename` parameter from a Content-Disposition header,
    /// preferring the RFC 5987 `filename*` form when present. Returns nil
    /// if no filename could be parsed.
    static func filenameFromContentDisposition(_ cd: String?) -> String? {
        guard let cd = cd, !cd.isEmpty else { return nil }
        // Tokens are separated by `;` and may carry quoted values. We do
        // a light pass — not a full RFC 6266 parser — sufficient for
        // typical browser servers (GitHub, Cloudflare, S3, etc.).
        let parts = cd.split(separator: ";")
        var asciiFallback: String? = nil
        for part in parts {
            let token = part.trimmingCharacters(in: .whitespaces)
            // RFC 5987 form: filename*=UTF-8''percent%20encoded.ext
            if token.lowercased().hasPrefix("filename*=") {
                let value = String(token.dropFirst("filename*=".count))
                let components = value.split(separator: "'", maxSplits: 2, omittingEmptySubsequences: false)
                if components.count == 3 {
                    let encoded = String(components[2])
                    if let decoded = encoded.removingPercentEncoding {
                        return basename(decoded)
                    }
                }
            } else if token.lowercased().hasPrefix("filename=") {
                var value = String(token.dropFirst("filename=".count))
                if value.hasPrefix("\"") && value.hasSuffix("\"") && value.count >= 2 {
                    value = String(value.dropFirst().dropLast())
                }
                if !value.isEmpty {
                    asciiFallback = basename(value)
                }
            }
        }
        return asciiFallback
    }

    private static func basename(_ path: String) -> String {
        // Strip any directory component a malicious server might include.
        var s = path
        if let slash = s.lastIndex(of: "/") {
            s = String(s[s.index(after: slash)...])
        }
        if let bslash = s.lastIndex(of: "\\") {
            s = String(s[s.index(after: bslash)...])
        }
        return s
    }

    private func uniqueDestination(for filename: String) -> URL {
        // String-concatenate the path rather than using `appendingPathComponent`,
        // which on Skip percent-encodes the name and stops `.path` round-tripping.
        // Use `-N` rather than the more idiomatic ` (N)` suffix: Skip's
        // `URL(fileURLWithPath:)` constructor throws on space + parens in
        // the path, and the throw is silently swallowed by the surrounding
        // detached `Task` — so a `Package (1).resolved` candidate would
        // kill the download mid-way without ever logging anything.
        let baseDir = downloadsDirectory.path
        let join: (String) -> String = { name in
            baseDir.hasSuffix("/") ? baseDir + name : baseDir + "/" + name
        }
        let firstPath = join(filename)
        if !FileManager.default.fileExists(atPath: firstPath) {
            return URL(fileURLWithPath: firstPath)
        }
        let dotIndex = filename.lastIndex(of: ".")
        let base: String
        let extDot: String
        if let dotIndex = dotIndex {
            base = String(filename[filename.startIndex..<dotIndex])
            extDot = String(filename[dotIndex..<filename.endIndex]) // includes the leading "."
        } else {
            base = filename
            extDot = ""
        }
        var counter = 1
        while counter < 1000 {
            let newName = "\(base)-\(counter)\(extDot)"
            let candidatePath = join(newName)
            if !FileManager.default.fileExists(atPath: candidatePath) {
                return URL(fileURLWithPath: candidatePath)
            }
            counter += 1
        }
        // Should never get here in practice — fall back to a timestamped name.
        let fallback = "\(base)-\(Int(Date().timeIntervalSince1970))\(extDot)"
        return URL(fileURLWithPath: join(fallback))
    }
}

// MARK: - Open completed files

/// Opens the downloaded file in the system's default handler.
@MainActor
func netSkipOpenDownloadedFile(_ item: NetSkipDownloadItem) {
    guard let local = item.localURL else { return }
    #if SKIP
    let ctx: Context = ProcessInfo.processInfo.androidContext
    let file = java.io.File(local.path)
    let authority = ctx.getPackageName() + ".fileprovider"
    var uri: Uri = Uri.fromFile(file)
    do {
        uri = FileProvider.getUriForFile(ctx, authority, file)
    } catch {
        // FileProvider authority isn't configured for this build — fall back
        // to a plain file:// URI. Pre-N this works; on newer Android it raises
        // FileUriExposedException, but reporting that beats silently no-op-ing.
        logger.log("FileProvider lookup failed for \(file.path): \(error)")
    }
    let intent = Intent(Intent.ACTION_VIEW)
    let resolverMime = ctx.getContentResolver().getType(uri)
    let mime = item.mimeType ?? resolverMime ?? "*/*"
    intent.setDataAndType(uri, mime)
    intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_ACTIVITY_NEW_TASK)
    // Wrap in a chooser so that (a) the user is always asked which app to
    // open the file with on first launch, and (b) emulators / devices that
    // have no app registered for the MIME type still show a clear system
    // dialog instead of the tap silently no-op-ing.
    let chooser = Intent.createChooser(intent, "Open file")
    chooser.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    do {
        ctx.startActivity(chooser)
    } catch {
        logger.log("Failed to open downloaded file: \(error)")
    }
    #else
    // iOS: use a UIDocumentInteractionController-backed UIActivityViewController.
    // This lets the user share or open the file in any registered handler.
    guard let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
          let root = scene.windows.first?.rootViewController else { return }
    let activity = UIActivityViewController(activityItems: [local], applicationActivities: nil)
    // Top-most presenter so Settings sheets etc. don't swallow the request.
    var presenter: UIViewController = root
    while let next = presenter.presentedViewController {
        presenter = next
    }
    if let pop = activity.popoverPresentationController {
        pop.sourceView = presenter.view
        pop.sourceRect = CGRect(x: presenter.view.bounds.midX,
                                y: presenter.view.bounds.midY,
                                width: 0, height: 0)
        pop.permittedArrowDirections = []
    }
    presenter.present(activity, animated: true)
    #endif
}

// MARK: - Number formatting helpers

func netSkipFormatBytes(_ bytes: Int64) -> String {
    if bytes <= 0 { return "0 B" }
    let units = ["B", "KB", "MB", "GB", "TB"]
    var value = Double(bytes)
    var idx = 0
    while value >= 1024.0 && idx < units.count - 1 {
        value /= 1024.0
        idx += 1
    }
    if idx == 0 {
        return "\(Int64(value)) \(units[idx])"
    }
    return String(format: "%.1f %@", value, units[idx])
}

func netSkipFormatRemaining(_ seconds: Double) -> String {
    if !seconds.isFinite || seconds < 0 { return "—" }
    let total = Int(seconds.rounded())
    if total < 60 {
        return "\(total)s"
    } else if total < 3600 {
        return "\(total / 60)m \(total % 60)s"
    } else {
        let h = total / 3600
        let m = (total % 3600) / 60
        return "\(h)h \(m)m"
    }
}

// MARK: - SwiftUI list

/// The Downloads sheet — lists in-flight and completed downloads.
@MainActor
public struct NetSkipDownloadsListView: View {
    @Environment(\.dismiss) private var dismiss
    private let manager: NetSkipDownloadManager

    public init(manager: NetSkipDownloadManager = .shared) {
        self.manager = manager
    }

    /// Whether any row is currently in a "finished" terminal state
    /// (completed, cancelled, or failed) — those are what `Clear` removes.
    private var hasFinishedDownloads: Bool {
        manager.downloads.contains(where: { !$0.isRunning })
    }

    public var body: some View {
        NavigationStack {
            content
                .navigationTitle(Text("Downloads", bundle: .module, comment: "downloads list nav title"))
                #if !os(macOS)
                .navigationBarTitleDisplayMode(.inline)
                #endif
                .toolbar {
                    if hasFinishedDownloads {
                        ToolbarItem(placement: .cancellationAction) {
                            Button {
                                manager.clearFinished()
                            } label: {
                                Text("Clear", bundle: .module, comment: "button on the Downloads sheet that removes every completed/cancelled/failed download from the list")
                            }
                            .accessibilityIdentifier("button.downloads.clearFinished")
                        }
                    }
                    ToolbarItem(placement: .automatic) {
                        Button {
                            dismiss()
                        } label: {
                            Text("Done", bundle: .module, comment: "downloads done button")
                                .bold()
                        }
                        .accessibilityIdentifier("button.downloads.done")
                    }
                }
        }
        // Auto-presented sheets feel less intrusive as half-sheets so the
        // page beneath is still partly visible. The user can still drag
        // up to .large for the full-screen view.
        .presentationDetents([.medium, .large])
    }

    @ViewBuilder
    private var content: some View {
        if manager.downloads.isEmpty {
            VStack(spacing: 16) {
                Image("download", bundle: .module)
                    .resizable()
                    .scaledToFit()
                    .frame(width: 48, height: 48)
                    .foregroundStyle(.secondary)
                Text("No downloads yet", bundle: .module, comment: "downloads empty state title")
                    .font(.title3)
                Text("Files you download from web pages will appear here.",
                     bundle: .module,
                     comment: "downloads empty state subtitle")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 32)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .accessibilityIdentifier("downloads.emptyState")
        } else {
            List {
                ForEach(manager.downloads) { item in
                    NetSkipDownloadRow(item: item, manager: manager)
                        .accessibilityIdentifier("downloads.row.\(item.filename)")
                }
            }
            .accessibilityIdentifier("downloads.list")
        }
    }
}

@MainActor
struct NetSkipDownloadRow: View {
    let item: NetSkipDownloadItem
    let manager: NetSkipDownloadManager

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(verbatim: item.filename)
                        .font(.body)
                        .lineLimit(1)
                        .accessibilityIdentifier("downloads.row.filename")
                    Text(verbatim: subtitle)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .accessibilityIdentifier("downloads.row.subtitle")
                }
                Spacer(minLength: 8)
                trailingControl
            }
            if item.isRunning && item.totalBytes > 0 {
                ProgressView(value: item.progress)
                    .accessibilityIdentifier("downloads.row.progress")
            } else if item.isRunning {
                ProgressView()
                    .accessibilityIdentifier("downloads.row.progress.indeterminate")
            }
            if case .failed(let message) = item.state {
                Text(verbatim: message)
                    .font(.caption2)
                    .foregroundStyle(.red)
                    .lineLimit(2)
            }
        }
        .padding(.vertical, 4)
        #if !SKIP
        .contentShape(Rectangle())
        #endif
    }

    @ViewBuilder
    private var trailingControl: some View {
        switch item.state {
        case .pending, .downloading:
            Button {
                manager.cancel(item)
            } label: {
                Image("close", bundle: .module)
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("downloads.row.cancel")
            .accessibilityLabel(Text("Cancel download", bundle: .module, comment: "accessibility label for cancel download button"))
        case .completed:
            HStack(spacing: 8) {
                Button {
                    netSkipOpenDownloadedFile(item)
                } label: {
                    Text("Open", bundle: .module, comment: "open downloaded file button")
                        .bold()
                }
                .buttonStyle(.borderless)
                .accessibilityIdentifier("downloads.row.open")
                Button {
                    manager.remove(item)
                } label: {
                    Image("close", bundle: .module)
                }
                .buttonStyle(.borderless)
                .accessibilityIdentifier("downloads.row.remove")
                .accessibilityLabel(Text("Remove from list", bundle: .module, comment: "accessibility label for removing a completed download from the list"))
            }
        case .cancelled, .failed:
            Button {
                manager.remove(item)
            } label: {
                Image("close", bundle: .module)
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("downloads.row.remove")
        }
    }

    private var subtitle: String {
        switch item.state {
        case .pending:
            return "Waiting…"
        case .downloading:
            let written = netSkipFormatBytes(item.bytesWritten)
            if item.totalBytes > 0 {
                let total = netSkipFormatBytes(item.totalBytes)
                if let remaining = item.estimatedRemainingSeconds {
                    return "\(written) of \(total) · \(netSkipFormatRemaining(remaining)) left"
                }
                return "\(written) of \(total)"
            }
            return written
        case .completed:
            return netSkipFormatBytes(item.totalBytes > 0 ? item.totalBytes : item.bytesWritten)
        case .cancelled:
            return "Cancelled"
        case .failed:
            return "Failed"
        }
    }
}
