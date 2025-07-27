# SIP (Safe Install Proxy) Technical Specification

> **Repository:** `sip/`  
> **Language:** Rust  
> **Binary:** `sip`  

---

## 1. Overview

**SIP** is a drop-in, cross-language CLI wrapper---

## 7. Core Workflows (Implemented)

### 7.0. `sip trust <package> --lang <lang> [--fetch] [--score <score>]`

**Status**: ✅ **implemented and working**

1. **Parse CLI** (`cli.rs` with `clap`)
2. **Load existing registry** for specified language
3. **If `--fetch` flag is present**:
   - query official registry (pypi.org or crates.io)
   - retrieve real version, hash, source url
   - validate against json schema
4. **Create PackageRecord** with:
   - real metadata (if fetched) or manual entry
   - specified trust score (default 0.0)
   - current date as last_reviewed
   - user endorsement
5. **Write updated registry** to disk
6. **Confirm addition** to user

```bash
# working examples:
sip trust requests --lang python --fetch --score 9.0
sip trust serde --lang rust --fetch --score 9.5
```

### 7.1. `sip install <n> [--lang L]` (planned)native package managers (pip, cargo, go). It enforces a "trusted registry" model: each package install request is checked against a curated metadata registry before the native installer is invoked. Unverified or low-trust packages trigger an interactive prompt (or fail an automated CI run).

This document describes:

- Core goals and threat model  
- High-level architecture and data flows  
- Detailed repo layout  
- Module responsibilities  
- Registry schema and config formats  
- CLI workflows per language  
- Extension points for new ecosystems  

---

## 2. Goals & Threat Model

### 2.1. Goals

1. **Prevent typosquatting**, dependency-confusion, and malicious build-scripts _before_ install.  
2. **Unify** the UX for Python, Rust, Go (and future ecosystems).  
3. **Enforce** trust thresholds via a signed, community-curated registry.  
4. **Minimize friction**: mirror native CLI syntax, support CI/CD fail-fast mode.  
5. **Provide extensibility**: easy to add new languages, custom trust policies.

### 2.2. Threats Addressed

- **Typosquatting**: installing "requets" instead of "requests"  
- **Dependency confusion**: private package name collision  
- **Malicious build scripts**: `setup.py`, `build.rs`, or `go generate` executes attacker code  
- **Unmaintained packages**: dropped-off or single-maintainer projects  

---

## 3. High-Level Architecture

```
┌──────────┐
│  user →  │
│  sip CLI │
└────┬─────┘
     │ parse args, detect language
┌────▼─────────────────────────┐
│  sip core (src/sip/)         │
│  • cli.rs  – dispatch        │
│  • verify.rs – lookup & vet  │
│  • registry.rs – load JSON   │
│  • prompt.rs – interactive   │
│  • runner.rs – invoke tool   │
└────┬─────────────────────────┘
     │
     │ if verified →
     │    spawn native installer
     │ else → prompt / abort
┌────▼─────────────────────────┐
│  langs (src/langs/)          │
│  • python.rs – `pip install` │
│  • rust.rs   – `cargo ...`   │
│  • go.rs     – `go install`  │
└──────────────────────────────┘
```

---

## 4. Repository Layout

```
sip/
├── Cargo.toml                  # Rust project manifest
├── README.md                   # High-level intro & quickstart
├── LICENSE
├── .github/
│   └── workflows/
│       └── ci.yml              # CI: fmt, clippy, tests
├── config/
│   └── sip.toml                # Global settings & thresholds
├── registry/
│   ├── schema/
│   │   └── sip-package.json    # JSON Schema for registry entries
│   └── data/
│       ├── python/
│       │   └── trusted-packages.json
│       ├── rust/
│       │   └── trusted-crates.json
│       └── go/
│           └── trusted-modules.json
├── scripts/
│   ├── update_registry.rs      # CLI to add / sign / bump entries
│   └── scan_package.rs         # Heuristic scanner (optional)
├── src/
│   ├── main.rs                 # Entrypoint → sip::runner::run()
│   ├── cli.rs                  # `clap` definitions & arg parsing
│   ├── sip/
│   │   ├── mod.rs              # `pub mod { verify, registry, prompt, runner }`
│   │   ├── verify.rs           # Registry lookup & trust scoring
│   │   ├── registry.rs         # Load & validate JSON manifest
│   │   ├── prompt.rs           # Interactive confirm / abort logic
│   │   └── runner.rs           # Glue: parse → verify → invoke
│   └── langs/
│       ├── mod.rs              # `pub mod { python, rust, go }`
│       ├── python.rs           # `fn install_python(...)`
│       ├── rust.rs             # `fn install_rust(...)`
│       └── go.rs               # `fn install_go(...)`
└── tests/
    ├── unit/                   # `verify.rs`, `registry.rs` tests
    └── integration/            # Full "sip install ..." scenarios
```

---

## 5. Registry Schema & Real Data Fetching

**`registry/schema/sip-package.json`** (implemented)

```jsonc
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SIP Package Entry",
  "type": "object",
  "properties": {
    "name":       { "type": "string" },
    "version":    { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+.*$" },
    "hash":       { "type": "string" },      // e.g. "sha256:..."
    "trust_score":{ "type": "number" },      // 0.0 – 10.0
    "endorsed_by":{ "type": "array", "items": { "type": "string" } },
    "last_reviewed": { "type": "string", "format": "date" },
    "source":     { "type": "string", "format": "uri" }
  },
  "required": [ "name", "version", "hash", "trust_score", "last_reviewed", "source" ]
}
```

### 5.1. Automatic Package Fetching

The `--fetch` flag enables automatic metadata retrieval from official registries:

**Python (PyPI)**:
- endpoint: `https://pypi.org/pypi/{package}/json`
- retrieves: latest version, sha256 hash, homepage/project url
- validates: ensures real semver and valid uri

**Rust (crates.io)**:
- endpoint: `https://crates.io/api/v1/crates/{package}`
- retrieves: max_version, checksum, crate url
- validates: ensures real semver and valid uri

**Example usage**:
```bash
sip trust requests --lang python --fetch --score 9.0
# fetches real metadata: version "2.32.4", real sha256, source url
```

Each language's registry (e.g. `trusted-packages.json`) is simply an **array** of these entries.

---

## 6. Configuration (`config/sip.toml`)

```toml
[registry]
# Path to local registry data (can be a Git submodule or remote sync)
path = "registry/data"

# Minimum trust score to auto-install without prompt
trust_threshold = 8.0

[prompt]
# Always confirm if trust_score < threshold
confirm_low_score = true
# Confirm unknown packages (not found in registry)
confirm_unknown = true

[lang.python]
binary = "pip"
args = ["install"]

[lang.rust]
binary = "cargo"
args = ["install"]

[lang.go]
binary = "go"
args = ["install", "-v"]
```

---

## 7. Core Workflows

### 7.1. `sip install <name> [--lang L]`

1. **Parse CLI** (`cli.rs` with `clap`)
2. **Determine language**:
   - Explicit via `--lang python|rust|go`
   - Implicit by project files (e.g. `Cargo.toml`, `requirements.txt`)
3. **Load config** (`config` crate)
4. **Load registry** (`registry.rs`)
5. **Lookup entry** (`verify.rs`):
   - If found & `trust_score ≥ threshold`: *allow*
   - If found & `trust_score < threshold`: *prompt*
   - If not found: *prompt* or *abort*
6. **Prompt user** (`prompt.rs`, `dialoguer`):
   - Show name, version, score, endorsements
   - `[Y/n]` decision
7. **Invoke native installer** (`runner.rs` → `langs/*.rs`):
   - e.g. `pip install requests==2.31.0`
   - Capture exit code, propagate errors

### 7.2. `sip verify <name>`

Skips install; only prints:

```text
Package: requests
Version: 2.31.0
Trust score: 9.2
Endorsed by: @kennethreitz, @psf
Last reviewed: 2025-07-26
```

---

## 8. Language Backends

### 8.1. Python (`src/langs/python.rs`)

- Query PyPI metadata if not in registry (optional fallback)
- Construct `pip install <name>==<version>`
- Use `std::process::Command` or `duct` crate

### 8.2. Rust (`src/langs/rust.rs`)

- Read `Cargo.toml` to detect version constraints (optional)
- Construct `cargo install <crate> --version <version>`
- No build-script execution until after vetting

### 8.3. Go (`src/langs/go.rs`)

- Support both `go install pkg@version` and `go get`
- Verify `sumdb` and module checksum DB

---

## 9. Extensibility

To add a new language (e.g. JavaScript/npm):

1. **Add** `src/langs/js.rs` with:
   - `fn install_js(name: &str, version: &str)`
2. **Register** in `langs/mod.rs` and `cli.rs`
3. **Add** `registry/data/js/trusted-packages.json`
4. **Update** `config/sip.toml` with `[lang.js]` section

---

## 10. CI/CD & Automation

**`.github/workflows/ci.yml`** should:

```yaml
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
      - name: Install dependencies
        run: cargo fetch
      - name: Format & Lint
        run: |
          cargo fmt -- --check
          cargo clippy -- -D warnings
      - name: Run tests
        run: cargo test -- --nocapture
```

You can add a `sip vet` step in your project's CI to **fail** if any unverified deps appear.

---

## 11. Security Considerations

- **Registry integrity**: sign your JSON manifests with Git tags or use [TUF](https://github.com/theupdateframework/tuf)
- **Hash verification**: compare downloaded package hash against registry entry
- **Credential safety**: never leak `HOME/.pypirc` or other secrets in prompts
- **Audit logs**: record all install attempts in `~/.sip/logs/`

---

## 12. Next Steps

1. **MVP scope**: implement Python backend + registry lookup + prompt
2. **Add Rust & Go backends**
3. **Write unit/integration tests** for verify logic
4. **Build CLI agent workflows** in your IDE (e.g. VS Code extension)
5. **Prototype registry update script** (`scripts/update_registry.rs`)
6. **Publish alpha**, gather community feedback

---

*End of Technical Specification*