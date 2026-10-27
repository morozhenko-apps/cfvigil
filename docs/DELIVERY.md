# Development and acceptance

CFVigil separates source verification, package verification, native usability and public release authorization.

## Acceptance boundaries

**Source:** Stable JavaScript, Python and Rust tests check the latest-attempt rules, provider failures, history, localization, credential boundaries and signed package metadata. The source CI workflow is started explicitly after completed code changes, not after each commit.

**Packages:** Linux DEB/AppImage and Windows NSIS are built from the same source revision. Package identity, metadata, dependencies and SHA-256 sidecars are verified independently. Candidate archives are not public releases.

**Native behavior:** Install, tray interactions, Settings, data persistence, read-only provider access, notifications and uninstall effects must be observed on each supported OS. Test results for one platform do not substitute for the other.

**Release:** An approved tag, protected signing environment and public checksums are required before any binary becomes an official download.

Individual CI histories and debug transcripts are available through version-control and workflow history. They must not be copied into Markdown as a second, stale status ledger.

See [PRODUCT.md](PRODUCT.md), [NATIVE_MACHINE_GATE.md](NATIVE_MACHINE_GATE.md), [qa/MODE_B_MATRIX.md](qa/MODE_B_MATRIX.md) and [DISTRIBUTION.md](DISTRIBUTION.md).
