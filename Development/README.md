# Development

The only product source tree: `apps/desktop` owns the desktop integration,
`engine` owns runtime/distribution contracts, and `tooling` owns developer commands
and verification. Retained legacy code stays with its consumers; moving it here
does not enable VMware or Sandboxie operations.

From the checkout root, use `DEV.cmd doctor`, `DEV.cmd check:frontend`,
`DEV.cmd check:rust`, `DEV.cmd bench:frontend`, `DEV.cmd bench:rust`,
`DEV.cmd check`, `DEV.cmd test`, or
`DEV.cmd package`. Direct npm commands use `npm --prefix Development run <script>`.
`DEV.cmd dev` launches the development GUI and requires an appropriate session.

Toolchains, downloads, caches, build output and package exports belong to the
sibling `workspace`, not this source directory. New developer Cargo output defaults
to `workspace/builds/desktop/target`; existing caches remain preserved in place.
The declared toolchain and version files live here. App and Data remain siblings
of Development and are never source or package inputs.

To build/export without refreshing the prepared App, run
`npm --prefix Development run package:windows -- --export-only` with the declared
toolchain and reviewed DLL source configured. No command installs or launches an
installer or Minecraft automatically.
