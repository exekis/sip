use crate::cli::{self, Commands, Language};
use std::path::Path;
use std::fs;

pub fn run() {
    let cli = cli::parse();

    match cli.command {
        Commands::Install {
            package,
            version,
            lang,
            yes,
            extra_args,
        } => {
            handle_install(package, version, lang, yes, extra_args);
        }
        Commands::Verify {
            package,
            version,
            lang,
        } => {
            handle_verify(package, version, lang);
        }
    }
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
) {
    println!("installing package: {}", package);
    if let Some(version) = &version {
        println!("version: {}", version);
    }
    
    let detected_lang = lang.or_else(detect_language);
    match detected_lang {
        Some(language) => {
            println!("language: {}", language);
        }
        None => {
            println!("could not detect language - please specify with --lang flag");
            println!("supported languages: python, rust, go");
            return;
        }
    }
    
    if yes {
        println!("auto-approve mode enabled");
    }
    if !extra_args.is_empty() {
        println!("extra args: {:?}", extra_args);
    }
    
    // todo: implement actual install logic
    // 1. load registry and config
    // 2. verify package trust score
    // 3. prompt user if needed
    // 4. invoke native package manager
}

fn handle_verify(package: String, version: Option<String>, lang: Option<Language>) {
    println!("verifying package: {}", package);
    if let Some(version) = &version {
        println!("version: {}", version);
    }
    
    let detected_lang = lang.or_else(detect_language);
    match detected_lang {
        Some(language) => {
            println!("language: {}", language);
        }
        None => {
            println!("could not detect language - please specify with --lang flag");
            println!("supported languages: python, rust, go");
            return;
        }
    }
    
    // todo: implement actual verify logic
    // 1. load registry
    // 2. lookup package and display trust info
}
