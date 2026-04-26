// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SeabubbleV2",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "IslandShared",
            targets: ["IslandShared"]),
        .executable(
            name: "IslandUI",
            targets: ["IslandUI"]),
        .executable(
            name: "IslandDaemon",
            targets: ["IslandDaemon"]),
        .executable(
            name: "IslandCLI",
            targets: ["IslandCLI"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-argument-parser", from: "1.3.0"),
    ],
    targets: [
        .target(
            name: "IslandShared",
            dependencies: []),
        .executableTarget(
            name: "IslandUI",
            dependencies: ["IslandShared"]),
        .executableTarget(
            name: "IslandDaemon",
            dependencies: ["IslandShared"]),
        .executableTarget(
            name: "IslandCLI",
            dependencies: [
                "IslandShared",
                .product(name: "ArgumentParser", package: "swift-argument-parser")
            ],
            resources: [
                .process("Resources")
            ])
    ]
)