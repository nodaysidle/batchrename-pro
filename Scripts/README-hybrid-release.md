# Hybrid release flow

Local Mac builds the DMG; CI only creates the GitHub Release with generated notes when a version tag is pushed. No notarize and no CI binary build for the release.

1. **Build the DMG locally** on the Mac mini (e.g. `scripts/package-macos.sh` or your usual Tauri package path).
2. **Tag and push** a version tag (`v*`), e.g. `git tag v0.2.0 && git push origin v0.2.0`.
3. **CI creates the release** (`.github/workflows/release-on-tag.yml`) with generated notes — no binaries.
4. **Attach the asset(s)** with `Scripts/attach-release-asset.sh`:

   ```bash
   Scripts/attach-release-asset.sh v0.2.0 ./path/to/BatchRename-Pro-0.2.0-aarch64.dmg
   # or multiple:
   Scripts/attach-release-asset.sh v0.2.0 ./path/to/BatchRename-Pro-0.2.0-aarch64.dmg ./path/to/BatchRename-Pro-0.2.0-x86_64.dmg
   ```

Existing Latest release is **v0.1.0**. This automation does not republish or replace it; leave existing empty-release history and old tags alone. A new `v*` tag is required for a new release.
