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

private struct FixtureFailure: Error {
    let status: String
}

// Private synthetic child receipt. The public summary contains no paths or PIDs.
private struct PathObservation: Codable {
    let pid: Int32
    let slot: String
    let environmentHome: Bool
    let foundationHome: Bool
    let applicationSupport: Bool
    let caches: Bool
    let temporary: Bool

    var summary: [String: Bool] {
        ["environmentHome": environmentHome, "foundationHome": foundationHome,
         "applicationSupport": applicationSupport, "caches": caches,
         "temporary": temporary]
    }

    var allPrivate: Bool { summary.values.allSatisfy { $0 } }
}

@main
@MainActor
struct LifecycleFixture {
    private static let prefix = "com.mivubi.lazytester.lifecycle-fixture."
    private static let rootPrefix = "lazytester-lifecycle-"
    private static let executableName = "LifecycleFixture"
    private static let slots = ["A", "B"]

    static func main() {
        guard geteuid() != 0 else { receipt("REFUSED"); exit(2) }
        let arguments = Array(CommandLine.arguments.dropFirst())
        // Only the self-created app copy runs the ordinary AppKit event loop.
        if let id = Bundle.main.bundleIdentifier, id.hasPrefix(prefix),
           UUID(uuidString: String(id.dropFirst(prefix.count))) != nil {
            guard arguments.isEmpty || (arguments.count == 2 &&
                  arguments[0] == "--profile-probe-child" && slots.contains(arguments[1]))
            else { exit(2) }
            let app = NSApplication.shared
            app.setActivationPolicy(.accessory)
            if !arguments.isEmpty {
                do { try observeProfile(slot: arguments[1]) }
                catch { exit(2) }
            }
            app.run()
            return
        }
        guard arguments.isEmpty || arguments == ["--profile-probe"] else {
            receipt("REFUSED"); exit(2)
        }
        do {
            let result = try run(pair: !arguments.isEmpty)
            receipt(result.status, profiles: result.profiles)
            if result.status != "SYNTHETIC_EXIT_OBSERVED" &&
               result.status != "SYNTHETIC_PRIVATE_PATHS_OBSERVED" { exit(1) }
        } catch let failure as FixtureFailure {
            receipt(failure.status)
            exit(1)
        } catch {
            receipt("FIXTURE_PREPARATION_FAILED")
            exit(1)
        }
    }

    private static func receipt(_ status: String, profiles: [[String: Bool]] = []) {
        var report: [String: Any] = [
            "schema": 1, "scope": "SYNTHETIC_APPKIT_LIFECYCLE",
            "status": status, "runtimeAdmission": false
        ]
        if !profiles.isEmpty {
            report["profiles"] = profiles
            report["preferencesIsolation"] = "NOT_OBSERVED"
            report["authenticationIsolation"] = "NOT_OBSERVED"
            report["minecraftIsolation"] = "NOT_VERIFIED"
        }
        guard let bytes = try? JSONSerialization.data(withJSONObject: report, options: .sortedKeys),
              let text = String(data: bytes, encoding: .utf8) else { exit(2) }
        print(text)
    }

    private static func wait(_ seconds: Double, until condition: () -> Bool) -> Bool {
        let deadline = ProcessInfo.processInfo.systemUptime + seconds
        while !condition() {
            if ProcessInfo.processInfo.systemUptime >= deadline { return false }
            RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05))
        }
        return true
    }

    private static func profileRoot(_ root: URL, _ slot: String) -> URL {
        root.appendingPathComponent("Profile-" + slot, isDirectory: true)
    }

    private static func observationFile(_ root: URL, _ slot: String) -> URL {
        root.appendingPathComponent("observation-" + slot + ".json")
    }

    private static func contained(_ candidate: URL?, in root: URL) -> Bool {
        guard let candidate = candidate, candidate.isFileURL else { return false }
        let path = candidate.standardizedFileURL.resolvingSymlinksInPath().path
        let boundary = root.standardizedFileURL.resolvingSymlinksInPath().path
        return path == boundary || path.hasPrefix(boundary + "/")
    }

    private static func observeProfile(slot: String) throws {
        // No caller-provided path. Derive it only from our fresh fixture bundle.
        let bundle = Bundle.main.bundleURL.resolvingSymlinksInPath()
        let root = bundle.deletingLastPathComponent()
        guard bundle.lastPathComponent == "LifecycleFixture.app",
              root.lastPathComponent.hasPrefix(rootPrefix),
              UUID(uuidString: String(root.lastPathComponent.dropFirst(rootPrefix.count))) != nil
        else { throw FixtureFailure(status: "INVALID_FIXTURE_ROOT") }
        let profile = profileRoot(root, slot)
        let fm = FileManager.default
        // Observe standard path resolution only. Never read or write preferences,
        // Keychain, application-support or cache contents; no game is involved.
        let home = ProcessInfo.processInfo.environment["HOME"].map {
            URL(fileURLWithPath: $0, isDirectory: true)
        }
        let observation = PathObservation(
            pid: getpid(), slot: slot,
            environmentHome: contained(home, in: profile),
            foundationHome: contained(URL(fileURLWithPath: NSHomeDirectory()), in: profile),
            applicationSupport: contained(fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first, in: profile),
            caches: contained(fm.urls(for: .cachesDirectory, in: .userDomainMask).first, in: profile),
            temporary: contained(fm.temporaryDirectory, in: profile))
        // Write only a new receipt at the fixed, fixture-owned path.
        try JSONEncoder().encode(observation).write(to: observationFile(root, slot),
                                                    options: .withoutOverwriting)
    }

    private static func readObservation(root: URL, slot: String,
                                        application: NSRunningApplication) throws -> PathObservation {
        let url = observationFile(root, slot)
        let values = try url.resourceValues(forKeys: [.isRegularFileKey, .isSymbolicLinkKey, .fileSizeKey])
        guard values.isRegularFile == true, values.isSymbolicLink == false,
              let size = values.fileSize, (1...4096).contains(size)
        else { throw FixtureFailure(status: "PATH_OBSERVATION_UNCONFIRMED") }
        let file = try FileHandle(forReadingFrom: url)
        defer { try? file.close() }
        guard let bytes = try file.read(upToCount: 4097), bytes.count == size,
              let observation = try? JSONDecoder().decode(PathObservation.self, from: bytes),
              observation.pid == application.processIdentifier, observation.slot == slot
        else { throw FixtureFailure(status: "PATH_OBSERVATION_UNCONFIRMED") }
        return observation
    }

    private static func exactLiveApplication(_ app: NSRunningApplication,
                                              id: String, bundle: URL, exe: URL) -> Bool {
        guard !app.isTerminated, app.processIdentifier > 0,
              app.processIdentifier != getpid(), app.bundleIdentifier == id,
              app.bundleURL?.resolvingSymlinksInPath().path == bundle.path,
              app.executableURL?.resolvingSymlinksInPath().path == exe.path,
              let current = NSRunningApplication(processIdentifier: app.processIdentifier),
              current.isEqual(app) else { return false }
        return true
    }

    private static func launch(id: String, root: URL, bundle: URL, exe: URL,
                               slot: String? = nil) throws -> NSRunningApplication {
        let result = LaunchResult()
        let config = NSWorkspace.OpenConfiguration()
        config.createsNewApplicationInstance = true
        config.allowsRunningApplicationSubstitution = false
        config.activates = false
        config.addsToRecentItems = false
        config.promptsUserIfNeeded = false
        if let slot = slot {
            let profile = profileRoot(root, slot)
            config.arguments = ["--profile-probe-child", slot]
            // Hypothesis for synthetic children only, not a Minecraft launch recipe.
            config.environment = ["HOME": profile.path,
                                  "TMPDIR": profile.appendingPathComponent("Temp", isDirectory: true).path]
        }
        NSWorkspace.shared.openApplication(at: bundle, configuration: config) { app, error in
            DispatchQueue.main.async {
                result.application = app
                result.failed = error != nil
                result.completed = true
            }
        }
        guard wait(20, until: { result.completed }), !result.failed,
              let original = result.application
        else { throw FixtureFailure(status: "LAUNCH_UNCONFIRMED") }
        guard wait(20, until: { original.isFinishedLaunching || original.isTerminated }),
              original.isFinishedLaunching, exactLiveApplication(original, id: id, bundle: bundle, exe: exe)
        else { throw FixtureFailure(status: "OWNERSHIP_UNCONFIRMED") }
        return original
    }

    private static func close(_ app: NSRunningApplication, id: String, bundle: URL, exe: URL) throws {
        guard exactLiveApplication(app, id: id, bundle: bundle, exe: exe)
        else { throw FixtureFailure(status: "OWNERSHIP_UNCONFIRMED") }
        guard app.terminate() || app.isTerminated
        else { throw FixtureFailure(status: "CLOSE_REQUEST_REFUSED") }
        guard wait(20, until: { app.isTerminated })
        else { throw FixtureFailure(status: "EXIT_UNCONFIRMED") }
    }

    private static func run(pair: Bool) throws -> (status: String, profiles: [[String: Bool]]) {
        let fm = FileManager.default
        let id = prefix + UUID().uuidString.lowercased()
        let root = fm.temporaryDirectory.resolvingSymlinksInPath()
            .appendingPathComponent(rootPrefix + UUID().uuidString, isDirectory: true)
        try fm.createDirectory(at: root, withIntermediateDirectories: false,
                               attributes: [.posixPermissions: 0o700])
        let bundle = root.appendingPathComponent("LifecycleFixture.app", isDirectory: true)
        let contents = bundle.appendingPathComponent("Contents", isDirectory: true)
        let macos = contents.appendingPathComponent("MacOS", isDirectory: true)
        try fm.createDirectory(at: macos, withIntermediateDirectories: true)
        guard let original = Bundle.main.executableURL else { throw CocoaError(.fileNoSuchFile) }
        let exe = macos.appendingPathComponent(executableName)
        try fm.copyItem(at: original.resolvingSymlinksInPath(), to: exe)
        let plist: [String: Any] = [
            "CFBundleIdentifier": id, "CFBundleExecutable": executableName,
            "CFBundleName": executableName, "CFBundlePackageType": "APPL",
            "CFBundleVersion": "1", "CFBundleShortVersionString": "1.0",
            "LSMinimumSystemVersion": "14.0", "LSUIElement": true
        ]
        let bytes = try PropertyListSerialization.data(fromPropertyList: plist, format: .xml, options: 0)
        try bytes.write(to: contents.appendingPathComponent("Info.plist"), options: .withoutOverwriting)
        guard NSRunningApplication.runningApplications(withBundleIdentifier: id).isEmpty
        else { throw FixtureFailure(status: "PREEXISTING_APPLICATION") }
        if !pair {
            let app = try launch(id: id, root: root, bundle: bundle, exe: exe)
            try close(app, id: id, bundle: bundle, exe: exe)
            return ("SYNTHETIC_EXIT_OBSERVED", [])
        }
        for slot in slots {
            let profile = profileRoot(root, slot)
            try fm.createDirectory(at: profile, withIntermediateDirectories: false,
                                   attributes: [.posixPermissions: 0o700])
            try fm.createDirectory(at: profile.appendingPathComponent("Temp", isDirectory: true),
                                   withIntermediateDirectories: false)
        }
        // Both instances use exactly the same bundle and identifier.
        let a = try launch(id: id, root: root, bundle: bundle, exe: exe, slot: "A")
        let b = try launch(id: id, root: root, bundle: bundle, exe: exe, slot: "B")
        guard exactLiveApplication(a, id: id, bundle: bundle, exe: exe),
              exactLiveApplication(b, id: id, bundle: bundle, exe: exe),
              a.processIdentifier != b.processIdentifier, !a.isEqual(b)
        else { throw FixtureFailure(status: "INDEPENDENT_INSTANCE_UNCONFIRMED") }
        guard wait(5, until: {
            slots.allSatisfy { fm.fileExists(atPath: observationFile(root, $0).path) }
        }) else { throw FixtureFailure(status: "PATH_OBSERVATION_UNCONFIRMED") }
        let observations = [
            try readObservation(root: root, slot: "A", application: a),
            try readObservation(root: root, slot: "B", application: b)
        ]
        guard exactLiveApplication(a, id: id, bundle: bundle, exe: exe),
              exactLiveApplication(b, id: id, bundle: bundle, exe: exe)
        else { throw FixtureFailure(status: "INDEPENDENT_INSTANCE_UNCONFIRMED") }
        try close(a, id: id, bundle: bundle, exe: exe)
        // Never report independent lifecycle if closing A also terminates B.
        guard exactLiveApplication(b, id: id, bundle: bundle, exe: exe)
        else { throw FixtureFailure(status: "PEER_EXIT_OR_IDENTITY_CHANGED") }
        try close(b, id: id, bundle: bundle, exe: exe)
        // Preserve temporary files on all outcomes; no automatic recursive deletion.
        let status = observations.allSatisfy { $0.allPrivate }
            ? "SYNTHETIC_PRIVATE_PATHS_OBSERVED" : "SYNTHETIC_SHARED_OR_UNRESOLVED_PATHS"
        return (status, observations.map { $0.summary })
    }
}
