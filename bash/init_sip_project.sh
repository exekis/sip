#!/usr/bin/env bash
set -euo pipefail

PROJECT_NAME="sip"

echo "Creating new Rust binary project: $PROJECT_NAME"
cargo new --bin "$PROJECT_NAME"
cd "$PROJECT_NAME"

echo "Setting up directories..."
mkdir -p src/sip
mkdir -p src/langs
mkdir -p registry/{schema,data/{python,rust,go}}
mkdir -p config
mkdir -p tests/{unit,integration}
mkdir -p scripts
mkdir -p .github/workflows

echo "Creating placeholder files..."
touch src/cli.rs
touch src/sip/{mod.rs,verify.rs,registry.rs,runner.rs,prompt.rs}
touch src/langs/{mod.rs,python.rs,rust.rs,go.rs}
touch registry/schema/sip-package.json
touch registry/data/python/trusted-packages.json
touch registry/data/rust/trusted-crates.json
touch registry/data/go/trusted-modules.json
touch config/sip.toml
touch scripts/{update_registry.rs,scan_package.rs}
touch .github/workflows/ci.yml
touch tests/unit/mod.rs
touch tests/integration/mod.rs

echo "Patching src/main.rs"
cat > src/main.rs <<'EOF'
mod cli;
mod sip;
mod langs;

fn main() {
    sip::runner::run();
}
EOF

echo "Adding placeholder runner in src/sip/runner.rs"
cat > src/sip/runner.rs <<'EOF'
pub fn run() {
    println!("Welcome to sip: Safe Install Proxy");
    // CLI dispatch will go here
}
EOF

echo "Creating README.md and LICENSE"
cat > README.md <<'EOF'
# sip

**sip** (Safe Install Proxy) is a secure, cross-language CLI tool for safely installing packages from curated, verified registries. It wraps native package managers like \`pip\`, \`cargo\`, and \`go\` to prevent supply chain attacks.

## Quickstart

\`\`\`sh
sip install requests --lang python
\`\`\`

## Goals

- Prevent typosquatting
- Enforce community trust before install
- Support Python, Rust, Go, and more

## Status

MVP bootstrap in progress.
EOF

touch LICENSE

echo "Configuring dependencies in Cargo.toml"
cat >> Cargo.toml <<'EOF'

[dependencies]
clap = { version = "4.4", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
reqwest = { version = "0.12", features = ["json"] }
dialoguer = "0.11"
indicatif = "0.17"
EOF

echo "Initialization complete. To begin development:"
echo "  cd sip"

