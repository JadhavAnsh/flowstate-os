// swift-tools-version: 6.2
import PackageDescription

let package = Package(
    name: "FlowStateAppleRuntime",
    platforms: [.macOS(.v26)],
    products: [
        .executable(name: "flowstate-apple-runtime", targets: ["FlowStateAppleRuntime"]),
    ],
    targets: [
        .executableTarget(
            name: "FlowStateAppleRuntime",
            path: "Sources/FlowStateAppleRuntime"
        ),
    ]
)

