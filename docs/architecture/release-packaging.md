# Release Packaging

Release packaging is rooted in `scripts/`. GitHub release workflows should call these scripts directly and upload the artifact paths they produce.

## Source of Truth

- macOS DMG: `scripts/build-dmg.sh`
- signed macOS DMG wrapper: `scripts/build-dmg-signed.sh`
- Windows installer: `scripts/build-setup.ps1`
- Windows installer definition: `scripts/installer/termarx.iss`
- Linux tarball and AppImage: `scripts/build-linux.sh`
- app icon generation: `scripts/generate-icon.sh`
- current release CI (unsigned macOS DMG, no Apple secrets): `.github/workflows/ci.yml`
- Developer ID signed release flow, kept from upstream and not used by `ci.yml` (skipped unless the repository variable `SIGNED_RELEASE` is `true`): `.github/workflows/release.yml`
- GitHub macOS signing setup (only needed for the signed flow): `docs/architecture/macos-release-signing.md`
- stable release finalization and AUR dispatch: `.github/workflows/finalize-stable-release.yml`

The app version used by packaging scripts comes from `crates/desktop_app/Cargo.toml` unless an explicit version is passed.

## Artifact Paths

- macOS DMG: `dist/Termarx-<version>-macos-<arch>[-signed].dmg` (`ci.yml` publishes the unsigned one; `release.yml` requires `-signed`)
- Windows setup: `target/dist/Termarx-<version>-windows-<arch>-Setup.exe`
- Linux tarball: `target/dist/Termarx-<version>-linux-<arch>.tar.gz`
- Linux AppImage: `target/dist/Termarx-<version>-linux-<arch>.AppImage`

Release workflows must upload these exact locations. If a packaging script changes its output path or file name, update this document, `justfile`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, and `scripts/check-boundaries.sh` in the same change.

## Local Entrypoints

```sh
just build-dmg -- --version 0.3.0 --arch arm64
just build-setup -- -Version 0.3.0 -Arch x64 -Target x86_64-pc-windows-msvc
./scripts/build-linux.sh --version 0.3.0 --arch x86_64 --target x86_64-unknown-linux-gnu
```

Use `scripts/build-dmg-signed.sh` when a Developer ID signing identity is required. Unsigned DMGs should use `scripts/build-dmg.sh` directly.

The current release path (`ci.yml`) ships an unsigned macOS DMG and needs no Apple credentials; users open it with right-click → Open on first launch. The signed flow in `release.yml` requires the Developer ID certificate and App Store Connect team API key described in [macOS release signing](macos-release-signing.md), and both macOS architectures must be signed and notarized before it attaches artifacts. Stable release finalization waits for those signed DMGs.

## Boundary Rules

- Keep packaging scripts in `scripts/`.
- Keep generated artifacts out of the repo and under `dist/` or `target/dist/`.
- Keep release CI aligned with the script outputs.
- Keep platform-specific installer definitions under `scripts/installer/` unless a platform needs a larger packaging tree.
- File-manager context-menu payloads belong in `scripts/file-manager/` and must be copied by `scripts/build-linux.sh`, `scripts/build-dmg.sh`, and `scripts/installer/termarx.iss`. Do not add a parallel `packaging/` tree for those files.

## Validation

Run these checks after packaging or release workflow changes:

```sh
bash -n scripts/build-dmg.sh scripts/build-dmg-signed.sh scripts/setup-macos-signing.sh scripts/build-linux.sh
pwsh -NoProfile -Command '$null = [System.Management.Automation.Language.Parser]::ParseFile("scripts/build-setup.ps1", [ref]$null, [ref]$null)' # when PowerShell is available
just check-boundaries
```

For a local GPUI release performance gate:

```sh
cargo build --release -p termy
./scripts/check-gpui-launch-idle.sh
```

The gate can seed `--plugins`, require `--expect-no-bun`, or load a fixture with
`--workspace-store` to cover plugin and persisted-session regressions.
