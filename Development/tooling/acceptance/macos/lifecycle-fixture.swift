// Explicit synthetic AppKit lifecycle acceptance fixture, not a game launcher.
// No product command, arbitrary app target, account, world, or profile access.
import AppKit
import Foundation
import Darwin
import ScreenCaptureKit
import CoreGraphics
import CoreFoundation
import Security

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
    let preferenceBefore: String?
    let preferenceAfter: String?

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
        NSBezierPath.fill(bounds)
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

// One fixture-only workspace. Snapshot views share the same two CGImages;
// no image encoding, background sampling, or input forwarding.
@MainActor
private final class WorkspacePanel: NSObject {
    let window: NSWindow
    private let active = NSImageView()
    private let previews = [NSImageView(), NSImageView()]
    private let status = NSTextField(labelWithString: "")
    private var buttons: [NSButton] = []
    private var images: [NSImage] = []
    private(set) var selection: Int?
    private(set) var selectedWhileActive = false

    override init() {
        window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 560, height: 340),
                          styleMask: [.titled, .closable], backing: .buffered, defer: false)
        super.init()
        window.title = "Lazy Tester — Synthetic Workspace"
        window.isReleasedWhenClosed = false
        let content = NSView(frame: NSRect(x: 0, y: 0, width: 560, height: 340))
        active.frame = NSRect(x: 152, y: 118, width: 256, height: 192)
        active.imageScaling = .scaleProportionallyUpOrDown
        content.addSubview(active)
        status.frame = NSRect(x: 12, y: 312, width: 536, height: 20)
        content.addSubview(status)
        for index in 0..<2 {
            let x = CGFloat(144 + 140 * index)
            previews[index].frame = NSRect(x: x, y: 36, width: 132, height: 80)
            previews[index].imageScaling = .scaleProportionallyUpOrDown
            content.addSubview(previews[index])
            let button = NSButton(title: index == 0 ? "Select A" : "Select B",
                                  target: self, action: #selector(selectSlot(_:)))
            button.tag = index
            button.frame = NSRect(x: x, y: 6, width: 132, height: 26)
            button.isEnabled = false
            buttons.append(button)
            content.addSubview(button)
        }
        window.contentView = content
        window.center()
    }

    func publish(_ frames: [CGImage], selected: Int) {
        precondition(frames.count == 2 && (0..<2).contains(selected))
        images = frames.map { NSImage(cgImage: $0, size: NSSize(width: CGFloat($0.width), height: CGFloat($0.height))) }
        for index in 0..<2 { previews[index].image = images[index] }
        active.image = images[selected]
    }

    func prepareSelection(_ expected: Int) {
        selection = nil
        selectedWhileActive = false
        status.stringValue = expected == 0 ? "Select A, then click its native window." :
                                            "Select B, then click its native window."
        buttons.forEach { $0.isEnabled = true }
    }

    @objc private func selectSlot(_ sender: NSButton) {
        guard selection == nil, images.count == 2, (0..<2).contains(sender.tag) else { return }
        selectedWhileActive = NSApp.isActive && window.isKeyWindow
        selection = sender.tag
        active.image = images[sender.tag]
        buttons.forEach { $0.isEnabled = false }
    }

    func clear() {
        active.image = nil
        previews.forEach { $0.image = nil }
        images.removeAll()
        buttons.forEach { $0.isEnabled = false }
    }
}

@MainActor
private final class CaptureResult<T> {
    var completed = false
    var value: T?
}

// Synthetic one-shot cross-user rendezvous. No game, pixels, input or
// credentials. The UID comes from the connected kernel socket; the session
// number is reported by the cooperating fixture, NOT bound to that peer by
// the kernel. This is deliberately insufficient for production admission.
private enum RendezvousProbe {
    private static let marker: UInt32 = 0x4c545231
    private static func refuse() -> FixtureFailure {
        FixtureFailure(status: "SYNTHETIC_RENDEZVOUS_REFUSED")
    }

    private static func graphicalSession(_ requested: SecuritySessionId) throws -> SecuritySessionId {
        var id = SecuritySessionId()
        var attributes = SessionAttributeBits()
        guard SessionGetInfo(requested, &id, &attributes) == 0,
              id != 0, id != callerSecuritySession,
              attributes.contains(.sessionHasGraphicAccess),
              !attributes.contains(.sessionIsRoot) else { throw refuse() }
        return id
    }

    private static func metadata(_ path: String) throws -> stat {
        var value = stat()
        guard lstat(path, &value) == 0 else { throw refuse() }
        return value
    }

    private static func nonblocking(_ fd: Int32) throws {
        let flags = fcntl(fd, F_GETFL)
        var yes: Int32 = 1
        guard flags >= 0, fcntl(fd, F_SETFL, flags | O_NONBLOCK) == 0,
              setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, &yes,
                         socklen_t(MemoryLayout<Int32>.size)) == 0 else { throw refuse() }
    }

    private static func ready(_ fd: Int32, _ event: Int16, _ deadline: Double) throws {
        let remaining = deadline - ProcessInfo.processInfo.systemUptime
        guard remaining > 0 else { throw refuse() }
        var item = pollfd(fd: fd, events: event, revents: 0)
        // No retry after interruption or timeout. A single total deadline.
        guard Darwin.poll(&item, 1, Int32(min(120_000, ceil(remaining * 1000)))) == 1,
              item.revents & event != 0,
              item.revents & Int16(POLLNVAL | POLLERR) == 0 else { throw refuse() }
    }

    private static func checkPeer(_ fd: Int32, _ expected: uid_t) throws {
        var uid: uid_t = 0
        var gid: gid_t = 0
        guard getpeereid(fd, &uid, &gid) == 0, uid == expected,
              uid != 0, uid != geteuid() else { throw refuse() }
    }

    private static func transfer(_ fd: Int32, _ bytes: inout [UInt8],
                                 writing: Bool, deadline: Double) throws {
        var done = 0
        while done < bytes.count {
            try ready(fd, Int16(writing ? POLLOUT : POLLIN), deadline)
            let remaining = bytes.count - done
            let n = bytes.withUnsafeMutableBytes { buffer -> Int in
                guard let base = buffer.baseAddress else { return -1 }
                if writing { return Darwin.write(fd, base.advanced(by: done), remaining) }
                return Darwin.read(fd, base.advanced(by: done), remaining)
            }
            guard n > 0, n <= remaining else { throw refuse() }
            done += n
        }
    }

    private static func record(_ id: SecuritySessionId) -> [UInt8] {
        [marker, UInt32(id)].flatMap { value in
            [UInt8(truncatingIfNeeded: value >> 24),
             UInt8(truncatingIfNeeded: value >> 16),
             UInt8(truncatingIfNeeded: value >> 8),
             UInt8(truncatingIfNeeded: value)]
        }
    }

    private static func exchange(_ fd: Int32, server: Bool, own: SecuritySessionId,
                                 expected: uid_t, deadline: Double) throws {
        try nonblocking(fd)
        try checkPeer(fd, expected)
        var local = record(own)
        var remote = [UInt8](repeating: 0, count: 8)
        if !server { try transfer(fd, &local, writing: true, deadline: deadline) }
        try transfer(fd, &remote, writing: false, deadline: deadline)
        func word(_ offset: Int) -> UInt32 {
            remote[offset..<offset+4].reduce(UInt32(0)) { ($0 << 8) | UInt32($1) }
        }
        let reported = SecuritySessionId(word(4))
        guard word(0) == marker, reported != 0, reported != callerSecuritySession,
              reported != own, try graphicalSession(reported) == reported,
              try graphicalSession(callerSecuritySession) == own else { throw refuse() }
        if server { try transfer(fd, &local, writing: true, deadline: deadline) }
        try checkPeer(fd, expected)
        guard try graphicalSession(callerSecuritySession) == own else { throw refuse() }
    }

    // Both users must explicitly launch the same source-built fixture.
    // Parent is a disposable, already-provisioned /Users/Shared directory,
    // owned by the listener, non-writable by other users. The selected
    // partner needs traversal and socket access; no ACL changes here.
    // Only a fresh fixed leaf is bound. Preserve it for manual cleanup
    // rather than unlink a potentially replaced filesystem object.
    static func run(_ args: [String]) throws {
        guard args.count == 4, args[0] == "--rendezvous-probe",
              ["listen", "connect"].contains(args[1]),
              let partner = uid_t(args[3]), partner != 0, partner != geteuid(),
              geteuid() != 0, getuid() == geteuid() else { throw refuse() }
        let server = args[1] == "listen"
        let dir = args[2]
        guard dir.hasPrefix("/Users/Shared/"),
              URL(fileURLWithPath: dir).standardizedFileURL.resolvingSymlinksInPath().path == dir
              else { throw refuse() }
        let parent = try metadata(dir)
        guard parent.st_mode & mode_t(S_IFMT) == mode_t(S_IFDIR),
              parent.st_uid == (server ? geteuid() : partner),
              parent.st_mode & mode_t(S_IWGRP | S_IWOTH) == 0 else { throw refuse() }
        let path = dir + "/lazytester-rendezvous.sock"
        let bytes = Array(path.utf8)
        var address = sockaddr_un()
        guard !bytes.contains(0), bytes.count < MemoryLayout.size(ofValue: address.sun_path)
              else { throw refuse() }
        address.sun_family = sa_family_t(AF_UNIX)
        address.sun_len = UInt8(MemoryLayout<sockaddr_un>.size)
        withUnsafeMutableBytes(of: &address.sun_path) { dest in
            dest.initializeMemory(as: UInt8.self, repeating: 0)
            for (i, byte) in bytes.enumerated() { dest[i] = byte }
        }
        let own = try graphicalSession(callerSecuritySession)
        let deadline = ProcessInfo.processInfo.systemUptime + 120
        let fd = Darwin.socket(AF_UNIX, SOCK_STREAM, 0)
        guard fd >= 0 else { throw refuse() }
        defer { Darwin.close(fd) }
        try nonblocking(fd)

        if server {
            var existing = stat()
            guard lstat(path, &existing) != 0, errno == ENOENT else { throw refuse() }
            // Process-local creation mask; directory is not modified.
            let oldMask = umask(0o007)
            let result = withUnsafePointer(to: &address) {
                $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                    Darwin.bind(fd, $0, socklen_t(MemoryLayout<sockaddr_un>.size))
                }
            }
            umask(oldMask)
            guard result == 0, Darwin.listen(fd, 1) == 0 else { throw refuse() }
            try ready(fd, Int16(POLLIN), deadline)
            let peer = Darwin.accept(fd, nil, nil)
            guard peer >= 0 else { throw refuse() }
            defer { Darwin.close(peer) }
            try exchange(peer, server: true, own: own, expected: partner, deadline: deadline)
        } else {
            let leaf = try metadata(path)
            guard leaf.st_mode & mode_t(S_IFMT) == mode_t(S_IFSOCK),
                  leaf.st_uid == partner else { throw refuse() }
            let result = withUnsafePointer(to: &address) {
                $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                    Darwin.connect(fd, $0, socklen_t(MemoryLayout<sockaddr_un>.size))
                }
            }
            if result != 0 {
                guard errno == EINPROGRESS else { throw refuse() }
                try ready(fd, Int16(POLLOUT), deadline)
                var error: Int32 = 0
                var length = socklen_t(MemoryLayout<Int32>.size)
                guard getsockopt(fd, SOL_SOCKET, SO_ERROR, &error, &length) == 0,
                      error == 0 else { throw refuse() }
            }
            try exchange(fd, server: false, own: own, expected: partner, deadline: deadline)
        }
    }

    static func receipt(_ observed: Bool) {
        let report: [String: Any] = [
            "schema": 1, "scope": "SYNTHETIC_RENDEZVOUS",
            "status": observed ? "COOPERATIVE_PEER_OBSERVED" : "REFUSED",
            "connectionTimePeerUidMatched": observed,
            "distinctReportedGraphicalSessionObserved": observed,
            "peerSessionBinding": "NOT_VERIFIED",
            "peerCodeIdentity": "NOT_VERIFIED",
            "virtualDesktop": "NOT_VERIFIED",
            "minecraft": "NOT_VERIFIED", "runtimeAdmission": false
        ]
        guard let data = try? JSONSerialization.data(withJSONObject: report, options: .sortedKeys),
              let text = String(data: data, encoding: .utf8) else { exit(2) }
        print(text)
    }
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
                  slots.contains(arguments[1])) ||
                  (arguments.count == 3 && arguments[0] == "--preferences-probe-child" &&
                   slots.contains(arguments[1]) && ["write", "read"].contains(arguments[2]))
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
                do {
                    try observeProfile(slot: arguments[1],
                                       preferencePhase: arguments.count == 3 ? arguments[2] : nil)
                }
                catch { exit(2) }
            }
            app.run()
            return
        }
        if arguments.first == "--rendezvous-probe" {
            do {
                try RendezvousProbe.run(arguments)
                RendezvousProbe.receipt(true)
            } catch {
                RendezvousProbe.receipt(false)
                exit(1)
            }
            return
        }
        guard arguments.isEmpty || arguments == ["--profile-probe"] ||
              arguments == ["--workspace-probe"] || arguments == ["--shared-workspace-probe"] ||
              arguments == ["--preferences-probe"] else {
            receipt("REFUSED"); exit(2)
        }
        do {
            let shared = arguments == ["--shared-workspace-probe"]
            let workspace = arguments == ["--workspace-probe"] || shared
            if workspace {
                guard #available(macOS 14.0, *), CGPreflightScreenCaptureAccess()
                else { throw FixtureFailure(status: "CAPTURE_PERMISSION_UNAVAILABLE") }
            }
            let result = try run(pair: !arguments.isEmpty, workspace: workspace, shared: shared,
                                 preferences: arguments == ["--preferences-probe"])
            receipt(result.status, profiles: result.profiles)
            if result.status != "SYNTHETIC_EXIT_OBSERVED" &&
               result.status != "SYNTHETIC_PRIVATE_PATHS_OBSERVED" &&
               result.status != "SYNTHETIC_WINDOW_MARKERS_OBSERVED" &&
               result.status != "SYNTHETIC_WORKSPACE_SELECTION_OBSERVED" &&
               result.status != "SYNTHETIC_PRIVATE_PREFERENCES_OBSERVED" { exit(1) }
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
        if status == "SYNTHETIC_WORKSPACE_SELECTION_OBSERVED" {
            report["capture"] = "SEQUENTIAL_WINDOW_SNAPSHOTS"
            report["selection"] = "SHARED_PANEL_TO_NATIVE_FOCUS"
            report["interaction"] = "MANUAL_NATIVE_WINDOW_CLICKS"
            report["embeddedGameplay"] = "NOT_VERIFIED"
            report["inactiveLoginSession"] = "NOT_VERIFIED"
            report["minecraft"] = "NOT_VERIFIED"
        }
        if status == "SYNTHETIC_WINDOW_MARKERS_OBSERVED" {
            report["capture"] = "SEQUENTIAL_WINDOW_SNAPSHOTS"
            report["interaction"] = "MANUAL_NATIVE_WINDOW_CLICKS"
            report["sharedWorkspace"] = "NOT_VERIFIED"
            report["inactiveLoginSession"] = "NOT_VERIFIED"
            report["minecraft"] = "NOT_VERIFIED"
        }
        if !profiles.isEmpty {
            report["profiles"] = profiles
            if status == "SYNTHETIC_PRIVATE_PREFERENCES_OBSERVED" {
                report["preferencesIsolation"] = "SYNTHETIC_DOMAIN_ONLY"
            } else if status == "SYNTHETIC_SHARED_OR_UNRESOLVED_PREFERENCES" {
                report["preferencesIsolation"] = "SHARED_OR_UNRESOLVED"
            } else {
                report["preferencesIsolation"] = "NOT_OBSERVED"
            }
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

    private static func observationFile(_ root: URL, _ slot: String,
                                        phase: String? = nil) -> URL {
        root.appendingPathComponent("observation-" + slot + (phase.map { "-" + $0 } ?? "") + ".json")
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

    // Only the fresh UUID fixture bundle's own current-user/any-host
    // preference domain. This can leave a tiny synthetic preference in the
    // real user's Library if HOME is ignored. Never read game or global
    // preferences, never touch Keychain, never reset an existing domain.
    private static func preferenceObservation(slot: String, phase: String) throws -> (String, String) {
        guard let id = Bundle.main.bundleIdentifier, id.hasPrefix(prefix),
              UUID(uuidString: String(id.dropFirst(prefix.count))) != nil,
              slots.contains(slot), ["write", "read"].contains(phase)
        else { throw FixtureFailure(status: "PREFERENCE_DOMAIN_UNCONFIRMED") }
        let domain = id as CFString
        let key = "LazyTesterSyntheticMarker" as CFString
        func sync() throws {
            guard CFPreferencesSynchronize(domain, kCFPreferencesCurrentUser, kCFPreferencesAnyHost)
            else { throw FixtureFailure(status: "PREFERENCE_SYNC_UNCONFIRMED") }
        }
        func observed() throws -> String {
            guard let value = CFPreferencesCopyValue(key, domain, kCFPreferencesCurrentUser, kCFPreferencesAnyHost)
            else { return "ABSENT" }
            guard let marker = value as? String, slots.contains(marker)
            else { throw FixtureFailure(status: "PREFERENCE_DOMAIN_UNCONFIRMED") }
            return marker
        }
        try sync()
        let before = try observed()
        if phase == "write" {
            // A is first. B may see A in a shared domain; allow only that
            // exact fixture marker, never replace unexpected/preexisting data.
            guard before == "ABSENT" || (slot == "B" && before == "A")
            else { throw FixtureFailure(status: "PREFERENCE_DOMAIN_UNCONFIRMED") }
            CFPreferencesSetValue(key, slot as CFString, domain,
                                  kCFPreferencesCurrentUser, kCFPreferencesAnyHost)
            try sync()
        }
        return (before, try observed())
    }

    private static func observeProfile(slot: String, preferencePhase: String? = nil) throws {
        let root = try fixtureRoot()
        let profile = profileRoot(root, slot)
        let fm = FileManager.default
        // Ordinary profile mode remains path-only. The explicit preferences
        // mode writes only its UUID fixture domain marker; no game data.
        let preference = try preferencePhase.map { try preferenceObservation(slot: slot, phase: $0) }
        let home = ProcessInfo.processInfo.environment["HOME"].map {
            URL(fileURLWithPath: $0, isDirectory: true)
        }
        let observation = PathObservation(
            pid: getpid(), slot: slot,
            environmentHome: contained(home, in: profile),
            foundationHome: contained(URL(fileURLWithPath: NSHomeDirectory()), in: profile),
            applicationSupport: contained(fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first, in: profile),
            caches: contained(fm.urls(for: .cachesDirectory, in: .userDomainMask).first, in: profile),
            temporary: contained(fm.temporaryDirectory, in: profile),
            preferenceBefore: preference?.0, preferenceAfter: preference?.1)
        // Write only a new receipt at the fixed, fixture-owned path.
        try JSONEncoder().encode(observation).write(to: observationFile(root, slot, phase: preferencePhase),
                                                    options: .withoutOverwriting)
    }

    private static func readObservation(root: URL, slot: String,
                                        application: NSRunningApplication,
                                        phase: String? = nil) throws -> PathObservation {
        let url = observationFile(root, slot, phase: phase)
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
    private static func captureMarker(_ window: SCWindow) throws -> (marker: String, image: CGImage) {
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
        return (marker, image)
    }

    @available(macOS 14.0, *)
    private static func capturePair(_ applications: [NSRunningApplication], id: String,
                                    bundle: URL, exe: URL, expectedIDs: [CGWindowID]?)
        throws -> (ids: [CGWindowID], markers: [String], frames: [CGImage]) {
        guard applications.count == slots.count,
              applications.allSatisfy({ exactLiveApplication($0, id: id, bundle: bundle, exe: exe) })
        else { throw FixtureFailure(status: "OWNERSHIP_UNCONFIRMED") }
        guard CGPreflightScreenCaptureAccess()
        else { throw FixtureFailure(status: "CAPTURE_PERMISSION_UNAVAILABLE") }
        let windows = try shareableWindows()
        var ids: [CGWindowID] = []
        var markers: [String] = []
        var frames: [CGImage] = []
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
            let frame = try captureMarker(window)
            markers.append(frame.marker)
            frames.append(frame.image)
            guard exactLiveApplication(application, id: id, bundle: bundle, exe: exe)
            else { throw FixtureFailure(status: "OWNERSHIP_UNCONFIRMED") }
        }
        guard Set(ids).count == slots.count else {
            throw FixtureFailure(status: "WINDOW_IDENTITY_UNCONFIRMED")
        }
        // These are sequential bounded snapshots, not simultaneous live streams.
        return (ids, markers, frames)
    }

    @available(macOS 14.0, *)
    private static func workspaceProbe(_ applications: [NSRunningApplication], id: String,
                                       root: URL, bundle: URL, exe: URL,
                                       shared: Bool) throws {
        let fm = FileManager.default
        let initial = try capturePair(applications, id: id, bundle: bundle, exe: exe, expectedIDs: nil)
        guard initial.markers == ["red", "blue"],
              slots.allSatisfy({ !fm.fileExists(atPath: clickFile(root, $0).path) })
        else { throw FixtureFailure(status: "WORKSPACE_BASELINE_UNCONFIRMED") }

        // Shared mode adds a snapshot consumer, not a new runtime or game adapter.
        let panel = shared ? WorkspacePanel() : nil
        defer {
            panel?.clear()
            panel?.window.close()
        }
        if let panel = panel {
            NSApplication.shared.setActivationPolicy(.regular)
            panel.publish(initial.frames, selected: 0)
            panel.window.makeKeyAndOrderFront(nil)
            // The operator chooses the panel; no forced foreground activation.
        }
        for (index, slot) in slots.enumerated() {
            if let panel = panel {
                panel.prepareSelection(index)
                guard wait(30, until: {
                    panel.selection != nil || !panel.window.isVisible ||
                    applications.contains(where: { $0.isTerminated })
                }), panel.selection == index, panel.selectedWhileActive,
                    panel.window.isVisible, NSApp.isActive,
                    !fm.fileExists(atPath: clickFile(root, slot).path),
                    exactLiveApplication(applications[index], id: id, bundle: bundle, exe: exe)
                else { throw FixtureFailure(status: "WORKSPACE_SELECTION_UNCONFIRMED") }
                // Request focus only in response to a user click in our panel.
                // AppKit may refuse; never retry or synthesize a game input event.
                let target = applications[index]
                guard target.activate(from: NSRunningApplication.current, options: []),
                      wait(5, until: {
                          target.isTerminated || NSWorkspace.shared.frontmostApplication?.isEqual(target) == true
                      }), exactLiveApplication(target, id: id, bundle: bundle, exe: exe),
                      NSWorkspace.shared.frontmostApplication?.isEqual(target) == true
                else { throw FixtureFailure(status: "WORKSPACE_FOCUS_UNCONFIRMED") }
            }
            // Physical user interaction only; no event injection.
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
                  index != 0 || !fm.fileExists(atPath: clickFile(root, "B").path),
                  !shared || NSWorkspace.shared.frontmostApplication?.isEqual(applications[index]) == true
            else { throw FixtureFailure(status: "WORKSPACE_MARKER_MISMATCH") }
            panel?.publish(snapshot.frames, selected: index)
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
                               slot: String? = nil, workspace: Bool = false,
                               preferencePhase: String? = nil) throws -> NSRunningApplication {
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
            config.arguments = preferencePhase.map { ["--preferences-probe-child", slot, $0] }
                ?? ["--profile-probe-child", slot]
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

    private static func run(pair: Bool, workspace: Bool, shared: Bool,
                            preferences: Bool) throws -> (status: String, profiles: [[String: Bool]]) {
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
        if preferences {
            // Reuse the same bundle, profile roots and retained launch/close
            // helpers. Ordered writes, then fresh process reads while the
            // peer remains alive; no timing guess or new runtime route.
            func start(_ slot: String, _ phase: String) throws -> (NSRunningApplication, PathObservation) {
                let app = try launch(id: id, root: root, bundle: bundle, exe: exe,
                                     slot: slot, preferencePhase: phase)
                guard wait(5, until: {
                    fm.fileExists(atPath: observationFile(root, slot, phase: phase).path)
                }), exactLiveApplication(app, id: id, bundle: bundle, exe: exe)
                else { throw FixtureFailure(status: "PREFERENCE_OBSERVATION_UNCONFIRMED") }
                return (app, try readObservation(root: root, slot: slot, application: app, phase: phase))
            }
            func distinct(_ a: NSRunningApplication, _ b: NSRunningApplication) throws {
                guard exactLiveApplication(a, id: id, bundle: bundle, exe: exe),
                      exactLiveApplication(b, id: id, bundle: bundle, exe: exe),
                      a.processIdentifier != b.processIdentifier, !a.isEqual(b)
                else { throw FixtureFailure(status: "INDEPENDENT_INSTANCE_UNCONFIRMED") }
            }
            let (a, aw) = try start("A", "write")
            let (b, bw) = try start("B", "write")
            try distinct(a, b)
            try close(a, id: id, bundle: bundle, exe: exe)
            guard exactLiveApplication(b, id: id, bundle: bundle, exe: exe)
            else { throw FixtureFailure(status: "PEER_EXIT_OR_IDENTITY_CHANGED") }
            let (ar, av) = try start("A", "read")
            try distinct(ar, b)
            try close(b, id: id, bundle: bundle, exe: exe)
            guard exactLiveApplication(ar, id: id, bundle: bundle, exe: exe)
            else { throw FixtureFailure(status: "PEER_EXIT_OR_IDENTITY_CHANGED") }
            let (br, bv) = try start("B", "read")
            try distinct(ar, br)
            try close(ar, id: id, bundle: bundle, exe: exe)
            guard exactLiveApplication(br, id: id, bundle: bundle, exe: exe)
            else { throw FixtureFailure(status: "PEER_EXIT_OR_IDENTITY_CHANGED") }
            try close(br, id: id, bundle: bundle, exe: exe)
            let reports = [("A", aw, av), ("B", bw, bv)].map { slot, write, read in
                var report = write.summary
                report["freshProfilePathsPrivate"] = read.allPrivate
                report["markerInitiallyAbsent"] = write.preferenceBefore == "ABSENT"
                report["ownMarkerAfterWrite"] = write.preferenceAfter == slot
                report["ownMarkerAfterReopen"] = read.preferenceBefore == slot && read.preferenceAfter == slot
                return report
            }
            let privatePreferences = reports.allSatisfy {
                $0["markerInitiallyAbsent"] == true &&
                $0["ownMarkerAfterWrite"] == true && $0["ownMarkerAfterReopen"] == true
            }
            // Path and preference evidence stay distinct. Positive markers
            // do not prove Keychain, authentication or Minecraft isolation.
            return (privatePreferences ? "SYNTHETIC_PRIVATE_PREFERENCES_OBSERVED" :
                    "SYNTHETIC_SHARED_OR_UNRESOLVED_PREFERENCES", reports)
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
            try workspaceProbe([a, b], id: id, root: root, bundle: bundle, exe: exe, shared: shared)
            try close(a, id: id, bundle: bundle, exe: exe)
            guard exactLiveApplication(b, id: id, bundle: bundle, exe: exe)
            else { throw FixtureFailure(status: "PEER_EXIT_OR_IDENTITY_CHANGED") }
            try close(b, id: id, bundle: bundle, exe: exe)
            return (shared ? "SYNTHETIC_WORKSPACE_SELECTION_OBSERVED" : "SYNTHETIC_WINDOW_MARKERS_OBSERVED", [])
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
