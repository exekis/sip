# sip

**sip** (Safe Install Proxy) is a secure, cross-language CLI tool for safely installing packages from curated, verified registries. It wraps native package managers like `pip`, `cargo`, and `go` to prevent supply chain attacks.

## Quickstart

```sh
sip install requests --lang python
```

## Goals

- Prevent typosquatting
- Enforce community trust before install
- Support Python, Rust, Go, and more

## Status

MVP bootstrap in progress.
