# Verification and release

CFVigil separates four independent gates:

1. **Source:** stable JavaScript, Python and Rust tests, including provider, security, localization and distribution contracts.
2. **Packages:** Linux DEB/AppImage and Windows NSIS built from one reviewed revision, with verified metadata and per-file SHA-256.
3. **Native:** real Windows and Linux install, tray, Settings, keyring, upgrades and uninstall checks.
4. **Publication:** approved version tag on the exact reviewed `main` commit, protected signing environment and verified public downloads.

Do not treat a successful package build as proof of native integration or a public artifact as verified before checksum readback. Commit-specific CI results belong in GitHub Actions, not in documentation.

See [product](PRODUCT.md), [desktop smoke](NATIVE_MACHINE_GATE.md), [QA matrix](qa/MODE_B_MATRIX.md) and [distribution](DISTRIBUTION.md).
