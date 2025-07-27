# sip

**sip** (Safe Install Proxy) is a secure, cross-language CLI tool for safely installing packages from curated, verified registries. It wraps native package managers like `pip`, `cargo`, and `go` to prevent supply chain attacks.

## Installation
```sh
git clone git@github.com:exekis/sip.git # or replace with https
cd sip
cargo build
cargo install --path .
```


## Quickstart

```sh
sip install requests --lang python
```

## Goals
- Have a community maintained repo of trusted packages and libraries, with a safety score
- Trusted packages repo, containing verified packages (verified users can vote for safe packages, this increases the trust score)
- `sip` stops and warns the users before installing any unverified packages
- Prevent typosquatting
- Support Python, Rust, Go, and more

## Status

MVP bootstrap in progress.
