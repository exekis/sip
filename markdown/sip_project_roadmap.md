# SIP Project Roadmap

## Phase 1: MVP Bootstrap

- [x] Scaffold Rust project and repo structure
- [ ] Implement CLI parsing (`sip install`, `sip verify`)
- [ ] Load and validate local registry JSON schema
- [ ] Python backend:
  - [ ] Registry lookup in `trusted-packages.json`
  - [ ] Wrap `pip install <name>==<version>`
- [ ] Interactive prompt on unknown or low-trust packages
- [ ] Unit tests for registry loader and verify logic
- [ ] Integration tests: end-to-end `sip install` scenarios
- [ ] CI setup: `cargo fmt`, `cargo clippy`, `cargo test`

## Phase 2: Multi-Language & Trust Enhancements

- [ ] Rust backend:
  - [ ] Registry lookup in `trusted-crates.json`
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