# SIP Project Roadmap

## Phase 1: MVP Bootstrap

- [x] Scaffold Rust project and repo structure
- [x] Implement CLI parsing (`sip install`, `sip verify`, `sip trust`)
- [x] Implement language auto-detection logic
- [x] Load and validate local registry JSON schema
- [x] **Real package fetching from official registries**:
  - [x] PyPI integration: fetch version, hash, source url
  - [x] crates.io integration: fetch version, checksum, crate url
  - [x] Schema validation for all fetched data
- [x] **Trust management command**: `sip trust --fetch` working end-to-end
- [ ] Python backend:
  - [x] Registry lookup in `trusted-packages.json`
  - [ ] Wrap `pip install <n>==<version>`
- [ ] Interactive prompt on unknown or low-trust packages
- [ ] Unit tests for registry loader and verify logic
- [ ] Integration tests: end-to-end `sip install` scenarios
- [ ] CI setup: `cargo fmt`, `cargo clippy`, `cargo test`oadmap


## Phase 2: Multi-Language & Trust Enhancements

- [ ] Rust backend:
  - [x] Registry lookup in `trusted-crates.json`
  - [x] Real metadata fetching from crates.io API
  - [ ] Wrap `cargo install <crate> --version <version>`
- [ ] Go backend:
  - [ ] Registry lookup in `trusted-modules.json`
  - [ ] Wrap `go install pkg@version` / `go get`
- [ ] Configurable trust threshold in `config/sip.toml`
- [ ] Support `sip vet` (scan project for unverified deps)
- [ ] Support `sip freeze` (export lockfile with metadata)
- [ ] Add hash/signature verification for downloaded artifacts
- [ ] Documentation: detailed CLI reference and developer guide

## Phase 3: Ecosystem & Automation

- [ ] IDE extensions (VS Code, JetBrains) for real-time vetting
- [ ] GitHub Action: fail PRs that introduce unverified packages
- [ ] Remote registry hosting and sync scripts
- [ ] Community dashboard for package endorsements and voting
- [ ] Audit logging of all `sip` actions to `~/.sip/logs/`
- [ ] Extend support to additional ecosystems (npm, Maven, etc.)
- [ ] Package and publish `sip` binaries for major platforms