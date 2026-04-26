// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SeabubbleV2",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "IslandUI",
            targets: ["IslandUI"]),
    ],
    targets: [
        .target(
            name: "IslandUI",
            dependencies: []),
    ]
)