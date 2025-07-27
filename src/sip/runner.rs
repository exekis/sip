use crate::cli::{self, Commands, Language};
use crate::sip::registry::Registry;
use crate::sip::package::PackageRecord;
use crate::sip::fetch::fetch_from_pypi;
use std::path::Path;
use std::fs;
use std::process::{Command, Stdio};
use chrono::Utc;

pub fn run() {
    let cli = cli::parse();

    let result = match cli.command {
        Commands::Install {
            package,
            version,
            lang,
            yes,
            extra_args,
        } => {
            handle_install(package, version, lang, yes, extra_args)
        }
        Commands::Verify {
            package,
            version,
            lang,
        } => {
            handle_verify(package, version, lang)
        }
        Commands::Trust { package, version, lang, fetch, score } => {
            handle_trust(package, version, lang, fetch, Some(score))
        }
        Commands::Untrust { package, lang } => {
            handle_untrust(package, lang)
        }
        Commands::List { lang } => {
            handle_list(lang)
        }
        Commands::BulkTrust { file, lang, score } => {
            handle_bulk_trust(file, lang, Some(score))
        }
    };

    if let Err(e) = result {
        println!("error: {}", e);
        std::process::exit(1);
    }
}

fn load_registry() -> Result<Registry, Box<dyn std::error::Error>> {
    // registry data is now embedded in the binary - completely portable!
    Registry::new()
}

fn detect_language() -> Option<Language> {
    // check for language-specific files in current directory
    if Path::new("Cargo.toml").exists() {
        return Some(Language::Rust);
    }
    
    if Path::new("requirements.txt").exists() || 
       Path::new("setup.py").exists() || 
       Path::new("pyproject.toml").exists() {
        return Some(Language::Python);
    }
    
    if Path::new("go.mod").exists() || Path::new("go.sum").exists() {
        return Some(Language::Go);
    }
    
    // check for common project structures
    if Path::new("src").is_dir() {
        // check if src contains rust files
        if let Ok(entries) = fs::read_dir("src") {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        return Some(Language::Rust);
                    }
                    if ext == "py" {
                        return Some(Language::Python);
                    }
                    if ext == "go" {
                        return Some(Language::Go);
                    }
                }
            }
        }
    }
    
    None
}

fn handle_install(
    package: String,
    version: Option<String>,
    lang: Option<Language>,
    yes: bool,
    extra_args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("installing package: {}", package);
    if let Some(version) = &version {
        println!("version: {}", version);
    }
    
    let detected_lang = lang.or_else(detect_language);
    let language = match detected_lang {
        Some(language) => {
            println!("language: {}", language);
            language
        }
        None => {
            return Err("could not detect language - please specify with --lang flag".into());
        }
    };
    
    // load registry and check if package is trusted
    let registry = load_registry()?;
    
    if let Some(trusted_package) = registry.lookup_package(&package, &language) {
        println!("✓ package '{}' is trusted", package);
        println!("  version: {}", trusted_package.version);
        println!("  trust score: {:.1}", trusted_package.trust_score);
        println!("  endorsed by: {}", trusted_package.endorsed_by.join(", "));
        println!("  last reviewed: {}", trusted_package.last_reviewed);
        
        // proceed with installation
        install_package(&package, &version, &language, &extra_args)?;
    } else {
        println!("⚠ package '{}' is not in trusted registry", package);
        println!("consider using 'sip trust {}' to add it to your trusted packages", package);
        
        if !yes {
            // ask user if they want to proceed
            println!("do you want to proceed with installation anyway? [y/N]");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
                println!("installation cancelled");
                return Ok(());
            }
        }
        
        println!("proceeding with untrusted installation...");
        install_package(&package, &version, &language, &extra_args)?;
    }
    
    Ok(())
}

fn handle_verify(
    package: String, 
    version: Option<String>, 
    lang: Option<Language>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("verifying package: {}", package);
    if let Some(version) = &version {
        println!("version: {}", version);
    }
    
    let detected_lang = lang.or_else(detect_language);
    let language = match detected_lang {
        Some(language) => {
            println!("language: {}", language);
            language
        }
        None => {
            return Err("could not detect language - please specify with --lang flag".into());
        }
    };
    
    let registry = load_registry()?;
    
    if let Some(trusted_package) = registry.lookup_package(&package, &language) {
        println!("✓ package '{}' is trusted", package);
        println!("  version: {}", trusted_package.version);
        println!("  trust score: {:.1}", trusted_package.trust_score);
        println!("  endorsed by: {}", trusted_package.endorsed_by.join(", "));
        println!("  last reviewed: {}", trusted_package.last_reviewed);
        println!("  source: {}", trusted_package.source);
    } else {
        println!("⚠ package '{}' is not in trusted registry", package);
    }
    
    Ok(())
}

fn handle_trust(
    package: String, 
    version: Option<String>, 
    lang: Option<Language>, 
    fetch: bool, 
    score: Option<f64>
) -> Result<(), Box<dyn std::error::Error>> {
    let detected_lang = lang.or_else(detect_language);
    let language = match detected_lang {
        Some(language) => language,
        None => {
            return Err("could not detect language - please specify with --lang flag".into());
        }
    };
    
    let mut registry = Registry::load_mutable()?;
    
    let package_entry = if fetch && language == Language::Python {
        // fetch metadata from pypi
        println!("fetching metadata for '{}' from pypi...", package);
        let mut fetched_entry = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(fetch_from_pypi(&package))?;
        
        // override trust score if provided
        if let Some(trust_score) = score {
            fetched_entry.trust_score = trust_score;
        }
        
        // add user endorsement
        fetched_entry.endorsed_by.push("user".to_string());
        
        fetched_entry
    } else {
        // manual entry
        let version = version.unwrap_or_else(|| "1.0.0".to_string());
        let trust_score = score.unwrap_or(5.0);
        
        PackageRecord {
            name: package.clone(),
            version,
            hash: "sha256:manual".to_string(),
            trust_score,
            endorsed_by: vec!["user".to_string()],
            last_reviewed: Utc::now().format("%Y-%m-%d").to_string(),
            source: "https://manual.entry".to_string(),
        }
    };
    
    registry.add_package(package_entry, &language);
    registry.save_to_disk()?;
    
    println!("✓ added '{}' to trusted {} packages", package, match language {
        Language::Python => "python",
        Language::Rust => "rust", 
        Language::Go => "go",
    });
    
    Ok(())
}

fn handle_untrust(package: String, lang: Option<Language>) -> Result<(), Box<dyn std::error::Error>> {
    let detected_lang = lang.or_else(detect_language);
    let language = match detected_lang {
        Some(language) => language,
        None => {
            return Err("could not detect language - please specify with --lang flag".into());
        }
    };
    
    let mut registry = Registry::load_mutable()?;
    
    if registry.remove_package(&package, &language) {
        registry.save_to_disk()?;
        println!("✓ removed '{}' from trusted {} packages", package, match language {
            Language::Python => "python",
            Language::Rust => "rust",
            Language::Go => "go",
        });
    } else {
        println!("⚠ package '{}' was not found in trusted {} packages", package, match language {
            Language::Python => "python", 
            Language::Rust => "rust",
            Language::Go => "go",
        });
    }
    
    Ok(())
}

fn handle_list(lang: Option<Language>) -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::load_mutable()?;
    
    let packages = registry.list_packages(lang.as_ref());
    
    if packages.is_empty() {
        println!("no trusted packages found");
        return Ok(());
    }
    
    match lang {
        Some(l) => println!("trusted {} packages:", match l {
            Language::Python => "python",
            Language::Rust => "rust", 
            Language::Go => "go",
        }),
        None => println!("all trusted packages:"),
    }
    
    for package in packages {
        println!("  {} {} (score: {:.1}) - {}", 
            package.name, 
            package.version, 
            package.trust_score,
            package.last_reviewed
        );
    }
    
    Ok(())
}

fn handle_bulk_trust(
    file_path: String,
    language: Language,
    score: Option<f64>
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(&file_path)?;
    let package_names: Vec<&str> = content.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    
    if package_names.is_empty() {
        println!("no packages found in file: {}", file_path);
        return Ok(());
    }
    
    println!("found {} packages to process", package_names.len());
    
    let mut registry = Registry::load_mutable()?;
    let mut success_count = 0;
    let mut error_count = 0;
    
    for (i, package_name) in package_names.iter().enumerate() {
        println!("({}/{}) processing: {}", i + 1, package_names.len(), package_name);
        
        let result = if language == Language::Python {
            // fetch from pypi
            match tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(fetch_from_pypi(package_name)) {
                Ok(mut entry) => {
                    if let Some(trust_score) = score {
                        entry.trust_score = trust_score;
                    }
                    entry.endorsed_by.push("bulk-import".to_string());
                    Ok(entry)
                }
                Err(e) => Err(e)
            }
        } else {
            // manual entry for rust/go
            let trust_score = score.unwrap_or(5.0);
            Ok(PackageRecord {
                name: package_name.to_string(),
                version: "1.0.0".to_string(),
                hash: "manual".to_string(),
                trust_score,
                endorsed_by: vec!["bulk-import".to_string()],
                last_reviewed: Utc::now().format("%Y-%m-%d").to_string(),
                source: "manual".to_string(),
            })
        };
        
        match result {
            Ok(package_entry) => {
                registry.add_package(package_entry, &language);
                success_count += 1;
                println!("  ✓ added {}", package_name);
            }
            Err(e) => {
                error_count += 1;
                println!("  ✗ failed {}: {}", package_name, e);
            }
        }
    }
    
    registry.save_to_disk()?;
    
    println!("\nbulk trust completed:");
    println!("  ✓ {} packages added", success_count);
    if error_count > 0 {
        println!("  ✗ {} packages failed", error_count);
    }
    
    Ok(())
}

fn install_package(
    package: &str, 
    version: &Option<String>, 
    language: &Language, 
    extra_args: &[String]
) -> Result<(), Box<dyn std::error::Error>> {
    match language {
        Language::Python => {
            let mut args = vec!["install"];
            let package_spec: String;
            
            if let Some(v) = version {
                package_spec = format!("{}=={}", package, v);
                args.push(&package_spec);
            } else {
                args.push(package);
            }
            
            // add extra args
            for arg in extra_args {
                args.push(arg);
            }
            
            println!("running: pip {}", args.join(" "));
            let output = Command::new("pip")
                .args(&args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?;
                
            if !output.status.success() {
                return Err("pip install failed".into());
            }
        }
        Language::Rust => {
            let mut args = vec!["add"];
            let package_spec: String;
            
            if let Some(v) = version {
                package_spec = format!("{}@{}", package, v);
                args.push(&package_spec);
            } else {
                args.push(package);
            }
            
            // add extra args
            for arg in extra_args {
                args.push(arg);
            }
            
            println!("running: cargo {}", args.join(" "));
            let output = Command::new("cargo")
                .args(&args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?;
                
            if !output.status.success() {
                return Err("cargo add failed".into());
            }
        }
        Language::Go => {
            let mut args = vec!["get"];
            let package_spec: String;
            
            if let Some(v) = version {
                package_spec = format!("{}@{}", package, v);
                args.push(&package_spec);
            } else {
                args.push(package);
            }
            
            // add extra args  
            for arg in extra_args {
                args.push(arg);
            }
            
            println!("running: go {}", args.join(" "));
            let output = Command::new("go")
                .args(&args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?;
                
            if !output.status.success() {
                return Err("go get failed".into());
            }
        }
    }
    
    Ok(())
}
