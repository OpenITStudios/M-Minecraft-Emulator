fn main() {
    // Direct "tauri build" must not package legacy VMware resources under the
    // retired installed product identity, even if someone bypasses npm scripts.
    if std::env::var("PROFILE").as_deref() == Ok("release")
        && std::env::var_os("CARGO_FEATURE_NATIVE_WORKSPACE_PREVIEW").is_none()
    {
        panic!("Legacy Windows release packaging is blocked. Use the separately isolated native preview build.");
    }
    // Opt-in release workspace must be tied to a DISTINCT preview identity.
    // Cargo cannot activate it with the blocked VMware package configuration.
    if std::env::var_os("CARGO_FEATURE_NATIVE_WORKSPACE_PREVIEW").is_some() {
        let config = std::env::var("TAURI_CONFIG").unwrap_or_else(|_| {
            panic!("native-workspace-preview requires a Tauri CLI configuration overlay")
        });
        let patch: serde_json::Value = serde_json::from_str(&config)
            .expect("TAURI_CONFIG must be a JSON configuration patch");
        assert_eq!(
            patch.get("identifier").and_then(serde_json::Value::as_str),
            Some("com.mivubi.minecraft.instancemanager.preview"),
            "Native preview feature requires the distinct preview application identity",
        );
        assert_eq!(
            patch.get("mainBinaryName").and_then(serde_json::Value::as_str),
            Some("minecraft-instance-manager-preview"),
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
                == Some(&serde_json::Value::Null),
            "Native preview must remove inherited legacy installer hooks",
        );
        let resources = patch.pointer("/bundle/resources")
            .and_then(serde_json::Value::as_object)
            .expect("Native preview must explicitly remove legacy package resources");
        for name in [
            "../../../engine/guest/windows/",
            "../../../tooling/acceptance/windows/",
            "../../../engine/distribution/release-channel.json",
            "../../../engine/distribution/package-contract.json",
            "../../../engine/distribution/install-lifecycle-policy.json",
            "../../../engine/distribution/test-upgrade.ps1",
            "../../../engine/distribution/write-payload-manifest.ps1",
        ] {
            assert_eq!(
                resources.get(name),
                Some(&serde_json::Value::Null),
                "Native preview must delete every inherited VMware package resource",
            );
        }
        assert_eq!(
            patch.pointer("/bundle/externalBin").and_then(serde_json::Value::as_array)
                .map(Vec::len),
            Some(0),
            "Native preview cannot bundle the legacy virtual-clients sidecar",
        );
    }
    tauri_build::build()
}
