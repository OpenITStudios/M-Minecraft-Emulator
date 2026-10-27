fn main() {
    let native_preview = std::env::var_os("CARGO_FEATURE_NATIVE_WORKSPACE_PREVIEW").is_some();
    let lazy_package = std::env::var_os("CARGO_FEATURE_LAZY_TESTER_PACKAGE").is_some();

    // Both native product/preview features are tied to reviewed Windows
    // process roles, Win32 window recovery and NSIS identities. Neither may
    // turn a diagnostic macOS/Linux shell into an apparent native game build.
    // The no-feature debug shell remains available for platform feasibility.
    if (native_preview || lazy_package)
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
    {
        panic!("Windows-native preview and Lazy Tester package features require a Windows target; unsupported platforms have diagnostic UI only");
    }
    // Direct "tauri build" must not package legacy VMware resources under the
    // retired installed product identity, even if someone bypasses npm scripts.
    if std::env::var("PROFILE").as_deref() == Ok("release")
        && !native_preview
    {
        panic!("Legacy Windows release packaging is blocked. Use the separately isolated native preview build.");
    }
    // Opt-in release workspace must be tied to a DISTINCT preview identity.
    // Cargo cannot activate it with the blocked VMware package configuration.
    if native_preview {
        let config = std::env::var("TAURI_CONFIG").unwrap_or_else(|_| {
            panic!("native-workspace-preview requires a Tauri CLI configuration overlay")
        });
        let patch: serde_json::Value = serde_json::from_str(&config)
            .expect("TAURI_CONFIG must be a JSON configuration patch");
        let lazy = lazy_package;
        let (identifier, binary, hooks) = if lazy {
            (Some("com.mivubi.lazytester"), Some("lazy-tester"),
                serde_json::Value::String("windows/lazy-tester-hooks.nsh".into()))
        } else {
            (Some("com.mivubi.minecraft.instancemanager.preview"),
                Some("minecraft-instance-manager-preview"), serde_json::Value::Null)
        };
        assert_eq!(
            patch.get("identifier").and_then(serde_json::Value::as_str),
            identifier,
            "Native preview feature requires the distinct preview application identity",
        );
        assert_eq!(
            patch.get("mainBinaryName").and_then(serde_json::Value::as_str),
            binary,
            "Native preview must not reuse the legacy executable filename",
        );
        assert_eq!(
            patch.pointer("/bundle/windows/nsis/installMode")
                .and_then(serde_json::Value::as_str),
            Some("currentUser"),
            "Native preview must never require a machine-wide installation",
        );
        assert!(
            patch.pointer("/bundle/windows/nsis/installerHooks")
                == Some(&hooks),
            "Native preview must remove inherited legacy installer hooks",
        );
        let resources = patch.pointer("/bundle/resources")
            .and_then(serde_json::Value::as_object)
            .expect("Native preview must explicitly remove legacy package resources");
        // Base config is payload-free. The native overlay owns every resource.
        if !lazy {
            assert!(resources.is_empty(), "Preview cannot bundle resource payloads");
        }
        if lazy {
            assert_eq!(patch.get("productName").and_then(serde_json::Value::as_str), Some("Lazy Tester"));
            for (source, target) in [
                ("package-resources/lazy-tester/bootstrap-compat.dll", "independent-bootstrap/bootstrap-compat.dll"),
                ("package-resources/lazy-tester/bootstrap-enroll.dll", "independent-bootstrap/bootstrap-enroll.dll"),
                ("../../../engine/distribution/lazy-tester-package.json", "distribution/lazy-tester-package.json"),
                ("package-resources/lazy-tester/en-US/", "en-US/"),
            ] {
                assert_eq!(resources.get(source).and_then(serde_json::Value::as_str), Some(target));
            }
        }
        assert_eq!(resources.len(), if lazy { 4 } else { 0 }, "Unexpected native package resources");
        assert_eq!(
            patch.pointer("/bundle/externalBin").and_then(serde_json::Value::as_array)
                .map(Vec::len),
            Some(0),
            "Native preview cannot bundle the legacy virtual-clients sidecar",
        );
    }
    tauri_build::build()
}
