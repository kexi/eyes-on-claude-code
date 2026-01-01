// swift-tools-version:5.7
import PackageDescription

let package = Package(
    name: "ClaudeMonitor",
    platforms: [.macOS(.v12)],
    targets: [
        .executableTarget(
            name: "ClaudeMonitor",
            path: "Sources/ClaudeMonitor"
        )
    ]
)
