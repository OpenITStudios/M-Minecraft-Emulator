// Legacy VMware/Guest-Agent packaging retired. Never stage its source, binaries,
// privileged helpers or guest resources. Keep the explicit failure sentinel for
// older scripts that still attempt this disabled route.
const LEGACY_PACKAGE_RELEASE_APPROVED = false;
if (!LEGACY_PACKAGE_RELEASE_APPROVED) {
  throw new Error('PACKAGE_BLOCKED_NATIVE_FEASIBILITY: legacy virtual-client/VMware packaging is retired; use the separate Lazy Tester package:windows route.');
}
