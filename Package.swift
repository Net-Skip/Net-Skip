// swift-tools-version: 5.9
// This is free software: you can redistribute and/or modify it
// under the terms of the GNU Lesser General Public License 3.0
// as published by the Free Software Foundation https://fsf.org

import PackageDescription

let package = Package(
    name: "netskip-app",
    defaultLocalization: "en",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "NetSkipApp", type: .dynamic, targets: ["NetSkip"]),
    ],
    dependencies: [
        .package(url: "https://source.skip.tools/skip.git", from: "0.8.19"),
        .package(url: "https://source.skip.tools/skip-foundation.git", from: "0.5.11"),
        .package(url: "https://source.skip.tools/skip-ui.git", from: "0.5.10"),
        .package(url: "https://source.skip.tools/skip-motion.git", from: "0.2.0"),
        .package(url: "https://source.skip.tools/skip-web.git", from: "0.1.0"),
        .package(url: "https://source.skip.tools/skip-model.git", from: "0.5.4"),
        .package(url: "https://source.skip.tools/skip-sql.git", from: "0.6.1"),
        .package(url: "https://source.skip.tools/skip-script.git", from: "0.4.1"),
        .package(url: "https://source.skip.tools/skip-xml.git", from: "0.1.2"),
        .package(url: "https://source.skip.tools/skip-zip.git", from: "0.3.0"),
    ],
    targets: [
        .target(name: "NetSkip", dependencies: [
            "NetSkipModel",
            .product(name: "SkipUI", package: "skip-ui"),
            .product(name: "SkipMotion", package: "skip-motion"),
            .product(name: "SkipWeb", package: "skip-web"),
        ], resources: [.process("Resources")], plugins: [.plugin(name: "skipstone", package: "skip")]),
        .testTarget(name: "NetSkipTests", dependencies: [
            "NetSkip",
            .product(name: "SkipTest", package: "skip")
        ], resources: [.process("Resources")], plugins: [.plugin(name: "skipstone", package: "skip")]),

        .target(name: "NetSkipModel", dependencies: [
            .product(name: "SkipModel", package: "skip-model"),
            .product(name: "SkipSQLPlus", package: "skip-sql"),
            .product(name: "SkipScript", package: "skip-script"),
            .product(name: "SkipXML", package: "skip-xml"),
            .product(name: "SkipZip", package: "skip-zip"),
        ], resources: [.process("Resources")], plugins: [.plugin(name: "skipstone", package: "skip")]),
        .testTarget(name: "NetSkipModelTests", dependencies: [
            "NetSkipModel",
        ], resources: [.process("Resources")], plugins: [.plugin(name: "skipstone", package: "skip")]),
    ]
)
