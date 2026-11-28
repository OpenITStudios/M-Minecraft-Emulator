// Explicit synthetic AppKit lifecycle acceptance fixture, not a game launcher.
// No product command, arbitrary app target, account, world, or profile access.
import AppKit
import Foundation
import Darwin
import ScreenCaptureKit
import CoreGraphics

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

// Synthetic visible content only. No generated input, foreign window control,
// accessibility permissions, or capture of the user's desktop.
@MainActor
private final class WorkspaceMarker: NSView {
    let slot: String
    let clickFile: URL
    private var clicked = false

    init(slot: String, clickFile: URL, frame: NSRect) {
        self.slot = slot
        self.clickFile = clickFile
        super.init(frame: frame)
    }

    required init?(coder: NSCoder) { nil }

    override func draw(_ dirtyRect: NSRect) {
        let color: NSColor = slot == "A"
            ? (clicked ? .green : .red)
            : (clicked ? .yellow : .blue)
        color.setFill()
        bounds.fill()
    }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }

    override func mouseDown(with event: NSEvent) {
        guard !clicked, window?.isKeyWindow == true else { return }
        clicked = true
        needsDisplay = true
        displayIfNeeded()
        // Only this synthetic marker is written. No input event leaves the view.
        try? Data([1]).write(to: clickFile, options: .withoutOverwriting)
    }
}

@MainActor
private final class CaptureResult<T> {
    var completed = false
    var value: T?
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
                  ["--profile-probe-child", "--workspace-probe-child"].contains(arguments[0]) &&
                  slots.contains(arguments[1]))
            else { exit(2) }
            let app = NSApplication.shared
            app.setActivationPolicy(.accessory)
            if arguments.first == "--workspace-probe-child" {
                do {
                    let window = try showWorkspaceWindow(slot: arguments[1])
                    withExtendedLifetime(window) { app.run() }
                } catch { exit(2) }
                return
            }
            if !arguments.isEmpty {
                do { try observeProfile(slot: arguments[1]) }
                catch { exit(2) }
            }
            app.run()
            return
        }
        guard arguments.isEmpty || arguments == ["--profile-probe"] ||
              arguments == ["--workspace-probe"] else {
            receipt("REFUSED"); exit(2)
        }
        do {
            let workspace = arguments == ["--workspace-probe"]
            if workspace {
                guard #available(macOS 14.0, *), CGPreflightScreenCaptureAccess()
                else { throw FixtureFailure(status: "CAPTURE_PERMISSION_UNAVAILABLE") }
            }
            let result = try run(pair: !arguments.isEmpty, workspace: workspace)
            receipt(result.status, profiles: result.profiles)
            if result.status != "SYNTHETIC_EXIT_OBSERVED" &&
               result.status != "SYNTHETIC_PRIVATE_PATHS_OBSERVED" &&
               result.status != "SYNTHETIC_WINDOW_MARKERS_OBSERVED" { exit(1) }
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
        if status == "SYNTHETIC_WINDOW_MARKERS_OBSERVED" {
            report["capture"] = "SEQUENTIAL_WINDOW_SNAPSHOTS"
            report["interaction"] = "MANUAL_NATIVE_WINDOW_CLICKS"
            report["sharedWorkspace"] = "NOT_VERIFIED"
            report["inactiveLoginSession"] = "NOT_VERIFIED"
            report["minecraft"] = "NOT_VERIFIED"
        }
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

    private static func fixtureRoot() throws -> URL {
        // No caller-provided path. Derive it only from our fresh fixture bundle.
        let bundle = Bundle.main.bundleURL.resolvingSymlinksInPath()
        let root = bundle.deletingLastPathComponent()
        guard bundle.lastPathComponent == "LifecycleFixture.app",
              root.lastPathComponent.hasPrefix(rootPrefix),
              UUID(uuidString: String(root.lastPathComponent.dropFirst(rootPrefix.count))) != nil
        else { throw FixtureFailure(status: "INVALID_FIXTURE_ROOT") }
        return root
    }

    private static func observeProfile(slot: String) throws {
        let root = try fixtureRoot()
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

    private static func clickFile(_ root: URL, _ slot: String) -> URL {
        root.appendingPathComponent("workspace-click-" + slot)
    }

    private static func showWorkspaceWindow(slot: String) throws -> NSWindow {
        let root = try fixtureRoot()
        guard let visible = NSScreen.main?.visibleFrame,
              visible.width >= 650, visible.height >= 260 else {
            throw FixtureFailure(status: "WORKSPACE_DISPLAY_UNAVAILABLE")
        }
        let width: CGFloat = 300
        let height: CGFloat = 200
        let x = visible.midX + (slot == "A" ? -width - 12 : 12)
        let window = NSWindow(contentRect: NSRect(x: x, y: visible.midY - height / 2,
                                                    width: width, height: height),
                              styleMask: [.titled, .closable], backing: .buffered, defer: false)
        window.title = "Lazy Tester Fixture " + slot
        window.isReleasedWhenClosed = false
        let marker = WorkspaceMarker(slot: slot, clickFile: clickFile(root, slot),
                                     frame: NSRect(x: 0, y: 0, width: width, height: height))
        marker.autoresizingMask = [.width, .height]
        window.contentView = marker
        window.makeKeyAndOrderFront(nil)
        return window
    }

    @available(macOS 14.0, *)
    private static func shareableWindows() throws -> [SCWindow] {
        let result = CaptureResult<SCShareableContent>()
        SCShareableContent.getExcludingDesktopWindows(true, onScreenWindowsOnly: true) { content, error in
            DispatchQueue.main.async {
                result.value = error == nil ? content : nil
                result.completed = true
            }
        }
        guard wait(10, until: { result.completed }), let content = result.value
        else { throw FixtureFailure(status: "WINDOW_ENUMERATION_UNCONFIRMED") }
        return content.windows
    }

    @available(macOS 14.0, *)
    private static func captureMarker(_ window: SCWindow) throws -> String {
        let result = CaptureResult<CGImage>()
        let config = SCStreamConfiguration()
        config.width = 320
        config.height = 240
        config.showsCursor = false
        config.ignoreShadowsSingleWindow = true
        // Capture exactly one verified fixture window, never a display.
        let filter = SCContentFilter(desktopIndependentWindow: window)
        SCScreenshotManager.captureImage(contentFilter: filter, configuration: config) { image, error in
            DispatchQueue.main.async {
                result.value = error == nil ? image : nil
                result.completed = true
            }
        }
        guard wait(10, until: { result.completed }), let image = result.value,
              image.width == config.width, image.height == config.height
        else { throw FixtureFailure(status: "WINDOW_CAPTURE_UNCONFIRMED") }
        let bitmap = NSBitmapImageRep(cgImage: image)
        var marker: String?
        // Sample five interior points; a blank, mixed, or unexpected image refuses.
        for (dx, dy) in [(0, 0), (-12, 0), (12, 0), (0, -12), (0, 12)] {
            guard let color = bitmap.colorAt(x: image.width / 2 + dx, y: image.height / 2 + dy)?
                .usingColorSpace(.deviceRGB), color.alphaComponent > 0.9
            else { throw FixtureFailure(status: "WINDOW_MARKER_UNCONFIRMED") }
            let rgb = [color.redComponent, color.greenComponent, color.blueComponent]
            let pattern = rgb.map { $0 > 0.65 ? "1" : ($0 < 0.35 ? "0" : "?") }.joined()
            let colors = ["100": "red", "001": "blue", "010": "green", "110": "yellow"]
            guard let observed = colors[pattern], marker == nil || marker == observed
            else { throw FixtureFailure(status: "WINDOW_MARKER_UNCONFIRMED") }
            marker = observed
        }
        guard let marker = marker else { throw FixtureFailure(status: "WINDOW_MARKER_UNCONFIRMED") }
        return marker
    }

    @available(macOS 14.0, *)
    private static func capturePair(_ applications: [NSRunningApplication], id: String,
                                    bundle: URL, exe: URL, expectedIDs: [CGWindowID]?)
        throws -> (ids: [CGWindowID], markers: [String]) {
        guard applications.count == slots.count,
              applications.allSatisfy({ exactLiveApplication($0, id: id, bundle: bundle, exe: exe) })
        else { throw FixtureFailure(status: "OWNERSHIP_UNCONFIRMED") }
        guard CGPreflightScreenCaptureAccess()
        else { throw FixtureFailure(status: "CAPTURE_PERMISSION_UNAVAILABLE") }
        let windows = try shareableWindows()
        var ids: [CGWindowID] = []
        var markers: [String] = []
        for (index, application) in applications.enumerated() {
            // Retained app identity plus exactly one owned window; title is
            // a fixture discriminator, not authority. Recheck before capture.
            let candidates = windows.filter {
                $0.owningApplication?.processID == application.processIdentifier
            }
            guard candidates.count == 1, let window = candidates.first,
                  window.title == "Lazy Tester Fixture " + slots[index],
                  window.windowID != 0,
                  expectedIDs == nil || expectedIDs?[index] == window.windowID,
                  exactLiveApplication(application, id: id, bundle: bundle, exe: exe)
            else { throw FixtureFailure(status: "WINDOW_IDENTITY_UNCONFIRMED") }
            ids.append(window.windowID)
            markers.append(try captureMarker(window))
            guard exactLiveApplication(application, id: id, bundle: bundle, exe: exe)
            else { throw FixtureFailure(status: "OWNERSHIP_UNCONFIRMED") }
        }
        guard Set(ids).count == slots.count else {
            throw FixtureFailure(status: "WINDOW_IDENTITY_UNCONFIRMED")
        }
        // These are sequential bounded snapshots, not simultaneous live streams.
        return (ids, markers)
    }

    @available(macOS 14.0, *)
    private static func workspaceProbe(_ applications: [NSRunningApplication], id: String,
                                       root: URL, bundle: URL, exe: URL) throws {
        let fm = FileManager.default
        let initial = try capturePair(applications, id: id, bundle: bundle, exe: exe, expectedIDs: nil)
        guard initial.markers == ["red", "blue"],
              slots.allSatisfy({ !fm.fileExists(atPath: clickFile(root, $0).path) })
        else { throw FixtureFailure(status: "WORKSPACE_BASELINE_UNCONFIRMED") }

        for (index, slot) in slots.enumerated() {
            // Physical user interaction only; no focus stealing or event injection.
            let message = "Click inside fixture window " + slot + " only (30 seconds).\n"
            FileHandle.standardError.write(Data(message.utf8))
            guard wait(30, until: {
                fm.fileExists(atPath: clickFile(root, slot).path) ||
                applications.contains(where: { $0.isTerminated })
            }) else { throw FixtureFailure(status: "MANUAL_INTERACTION_UNCONFIRMED") }
            let snapshot = try capturePair(applications, id: id, bundle: bundle, exe: exe,
                                           expectedIDs: initial.ids)
            let expected = index == 0 ? ["green", "blue"] : ["green", "yellow"]
            guard snapshot.markers == expected,
                  fm.fileExists(atPath: clickFile(root, slot).path),
                  index != 0 || !fm.fileExists(atPath: clickFile(root, "B").path)
            else { throw FixtureFailure(status: "WORKSPACE_MARKER_MISMATCH") }
        }
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
                               slot: String? = nil, workspace: Bool = false) throws -> NSRunningApplication {
        let result = LaunchResult()
        let config = NSWorkspace.OpenConfiguration()
        config.createsNewApplicationInstance = true
        config.allowsRunningApplicationSubstitution = false
        config.activates = false
        config.addsToRecentItems = false
        config.promptsUserIfNeeded = false
        if let slot = slot, workspace {
            config.arguments = ["--workspace-probe-child", slot]
        } else if let slot = slot {
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

    private static func run(pair: Bool, workspace: Bool) throws -> (status: String, profiles: [[String: Bool]]) {
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
        for slot in slots where !workspace {
            let profile = profileRoot(root, slot)
            try fm.createDirectory(at: profile, withIntermediateDirectories: false,
                                   attributes: [.posixPermissions: 0o700])
            try fm.createDirectory(at: profile.appendingPathComponent("Temp", isDirectory: true),
                                   withIntermediateDirectories: false)
        }
        // Both instances use exactly the same bundle and identifier.
        let a = try launch(id: id, root: root, bundle: bundle, exe: exe, slot: "A", workspace: workspace)
        let b = try launch(id: id, root: root, bundle: bundle, exe: exe, slot: "B", workspace: workspace)
        guard exactLiveApplication(a, id: id, bundle: bundle, exe: exe),
              exactLiveApplication(b, id: id, bundle: bundle, exe: exe),
              a.processIdentifier != b.processIdentifier, !a.isEqual(b)
        else { throw FixtureFailure(status: "INDEPENDENT_INSTANCE_UNCONFIRMED") }
        if workspace {
            guard #available(macOS 14.0, *)
            else { throw FixtureFailure(status: "WORKSPACE_OS_UNSUPPORTED") }
            try workspaceProbe([a, b], id: id, root: root, bundle: bundle, exe: exe)
            try close(a, id: id, bundle: bundle, exe: exe)
            guard exactLiveApplication(b, id: id, bundle: bundle, exe: exe)
            else { throw FixtureFailure(status: "PEER_EXIT_OR_IDENTITY_CHANGED") }
            try close(b, id: id, bundle: bundle, exe: exe)
            return ("SYNTHETIC_WINDOW_MARKERS_OBSERVED", [])
        }
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
