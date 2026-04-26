// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SecurityIsland",
    platforms: [
        .macOS(.v13) // macOS Ventura+ for modern SwiftUI and Swift concurrency
    ],
    products: [
        .executable(name: "SecurityIslandApp", targets: ["SecurityIslandApp"]),
        .library(name: "SecurityIslandCore", targets: ["SecurityIslandCore"]),
        .library(name: "SecurityIslandUI", targets: ["SecurityIslandUI"])
    ],
    targets: [
        .executableTarget(
            name: "SecurityIslandApp",
            dependencies: ["SecurityIslandCore", "SecurityIslandUI"]
        ),
        .target(
            name: "SecurityIslandCore",
            dependencies: []
        ),
        .target(
            name: "SecurityIslandUI",
            dependencies: ["SecurityIslandCore"]
        )
    ]
)
