// Explicit synthetic AppKit lifecycle acceptance fixture, not a game launcher.
// No product command, arbitrary app target, account, world, or profile access.
import AppKit
import Foundation
import Darwin

@MainActor
private final class LaunchResult {
    var completed = false
    var application: NSRunningApplication?
    var failed = false
}

@main
@MainActor
struct LifecycleFixture {
    private static let prefix = "com.mivubi.lazytester.lifecycle-fixture."
    private static let executableName = "LifecycleFixture"

    static func main() {
        // Only the self-created app copy runs an ordinary AppKit event loop.
        if let id = Bundle.main.bundleIdentifier, id.hasPrefix(prefix),
           UUID(uuidString: String(id.dropFirst(prefix.count))) != nil {
            guard CommandLine.arguments.count == 1 else { exit(2) }
            let app = NSApplication.shared
            app.setActivationPolicy(.accessory)
            app.run()
            return
        }
        guard CommandLine.arguments.count == 1, geteuid() != 0 else {
            receipt("REFUSED")
            exit(2)
        }
        do {
            let result = try run()
            receipt(result)
            if result != "SYNTHETIC_EXIT_OBSERVED" { exit(1) }
        } catch {
            // Do not print paths, OS diagnostics, process IDs or account data.
            receipt("FIXTURE_PREPARATION_FAILED")
            exit(1)
        }
    }

    private static func receipt(_ status: String) {
        // These statuses are only for this synthetic app, never Minecraft.
        print("{\"schema\":1,\"scope\":\"SYNTHETIC_APPKIT_LIFECYCLE\",\"status\":\"\(status)\",\"runtimeAdmission\":false}")
    }

    private static func wait(_ seconds: Double, until condition: () -> Bool) -> Bool {
        // Pump the main run loop: AppKit observations change as it runs.
        // Use monotonic uptime for the bounded wait, not wall-clock time.
        let deadline = ProcessInfo.processInfo.systemUptime + seconds
        while !condition() {
            if ProcessInfo.processInfo.systemUptime >= deadline { return false }
            RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05))
        }
        return true
    }

    private static func run() throws -> String {
        let fm = FileManager.default
        let id = prefix + UUID().uuidString.lowercased()
        let root = fm.temporaryDirectory.resolvingSymlinksInPath()
            .appendingPathComponent("lazytester-lifecycle-" + UUID().uuidString, isDirectory: true)
        // A fresh private directory. Never reuse a supplied or installed app.
        try fm.createDirectory(at: root, withIntermediateDirectories: false,
                               attributes: [.posixPermissions: 0o700])
        let bundle = root.appendingPathComponent("LifecycleFixture.app", isDirectory: true)
        let contents = bundle.appendingPathComponent("Contents", isDirectory: true)
        let macos = contents.appendingPathComponent("MacOS", isDirectory: true)
        try fm.createDirectory(at: macos, withIntermediateDirectories: true)
        guard let original = Bundle.main.executableURL else {
            throw CocoaError(.fileNoSuchFile)
        }
        let exe = macos.appendingPathComponent(executableName)
        try fm.copyItem(at: original.resolvingSymlinksInPath(), to: exe)
        let plist: [String: Any] = [
            "CFBundleIdentifier": id,
            "CFBundleExecutable": executableName,
            "CFBundleName": executableName,
            "CFBundlePackageType": "APPL",
            "CFBundleVersion": "1",
            "CFBundleShortVersionString": "1.0",
            "LSMinimumSystemVersion": "14.0",
            "LSUIElement": true
        ]
        let bytes = try PropertyListSerialization.data(fromPropertyList: plist,
                                                         format: .xml, options: 0)
        try bytes.write(to: contents.appendingPathComponent("Info.plist"), options: .withoutOverwriting)

        // Refuse adoption. The fresh identifier and bundle path are fixture-only.
        guard NSRunningApplication.runningApplications(withBundleIdentifier: id).isEmpty else {
            return "PREEXISTING_APPLICATION"
        }
        let result = LaunchResult()
        let config = NSWorkspace.OpenConfiguration()
        config.createsNewApplicationInstance = true
        config.allowsRunningApplicationSubstitution = false
        config.activates = false
        config.addsToRecentItems = false
        NSWorkspace.shared.openApplication(at: bundle, configuration: config) { app, error in
            DispatchQueue.main.async {
                result.application = app
                result.failed = error != nil
                result.completed = true
            }
        }
        // Never retry an uncertain launch or target a process by PID alone.
        guard wait(20, until: { result.completed }) else { return "LAUNCH_UNCONFIRMED" }
        guard !result.failed, let originalApp = result.application else { return "LAUNCH_UNCONFIRMED" }

        func exactLiveApplication() -> Bool {
            guard !originalApp.isTerminated,
                  originalApp.processIdentifier > 0,
                  originalApp.processIdentifier != getpid(),
                  originalApp.bundleIdentifier == id,
                  originalApp.bundleURL?.resolvingSymlinksInPath().path == bundle.path,
                  originalApp.executableURL?.resolvingSymlinksInPath().path == exe.path,
                  let current = NSRunningApplication(processIdentifier: originalApp.processIdentifier),
                  current.isEqual(originalApp) else { return false }
            return true
        }

        guard wait(20, until: { originalApp.isFinishedLaunching || originalApp.isTerminated }),
              originalApp.isFinishedLaunching, exactLiveApplication() else {
            return "OWNERSHIP_UNCONFIRMED"
        }
        // Request normal termination once, retaining the original app object.
        // A true return value means the request was sent, not that it exited.
        guard originalApp.terminate() else {
            return originalApp.isTerminated ? "SYNTHETIC_EXIT_OBSERVED" : "CLOSE_REQUEST_REFUSED"
        }
        guard wait(20, until: { originalApp.isTerminated }) else {
            return "EXIT_UNCONFIRMED"
        }
        // Preserve the disposable bundle on all outcomes. A later authorized
        // operator may remove it; there is no automatic recursive deletion.
        return "SYNTHETIC_EXIT_OBSERVED"
    }
}
