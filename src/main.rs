mod cli;
mod sip;

use sip::fetch::{fetch_from_pypi, fetch_from_crates};
use sip::package::PackageRecord;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = clap::Command::new("sip")
        .subcommand_required(true)
        .subcommand(
          clap::Command::new("trust")
            .arg(clap::arg!([pkg]))
            .arg(clap::arg!(--lang <LANG>))
            .arg(clap::arg!(--fetch))
            .arg(clap::arg!(--score <SCORE>).default_value("0.0"))
        )
        .get_matches();

    if let Some(m) = matches.subcommand_matches("trust") {
        let pkg = m.get_one::<String>("pkg").unwrap();
        let lang = m.get_one::<String>("lang").unwrap().as_str();
        let score: f64 = m.get_one::<String>("score").unwrap().parse()?;
        let fetch_flag = m.get_flag("fetch");

        // Only proceed with --fetch for now, as per the requirement
        if !fetch_flag {
            anyhow::bail!("--fetch flag is required to ensure real semver and valid URI");
        }

        // load existing registry, or start empty Vec if file missing
        let path = format!("registry/data/{}/trusted-{}.json",
                           lang,
                           if lang=="python" { "packages" } else { "crates" });
        let mut registry: Vec<PackageRecord> = match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s)?,
            Err(_) => Vec::new(),
        };

        // fetch real metadata
        let mut rec = match lang {
            "python" => fetch_from_pypi(pkg).await?,
            "rust"   => fetch_from_crates(pkg).await?,
            _ => anyhow::bail!("unsupported --lang (supported: python, rust)"),
        };

        // override trust_score
        rec.trust_score = score;
        registry.push(rec);

        // write back
        std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap())?;
        let out = serde_json::to_string_pretty(&registry)?;
        std::fs::write(&path, out)?;

        println!("✓ added {} to {}", pkg, path);
    }

    Ok(())
}
