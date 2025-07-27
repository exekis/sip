use crate::cli::{self, Commands, Language};
use crate::sip::registry::Registry;
use crate::sip::verify;
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
) {
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
            println!("could not detect language - please specify with --lang flag");
            println!("supported languages: python, rust, go");
            return;
        }
    };
    
    // load registry and verify package
    let registry = match load_registry() {
        Ok(reg) => reg,
        Err(e) => {
            println!("failed to load registry: {}", e);
            return;
        }
    };
    
    let trust_threshold = 8.0; // todo: load from config
    match verify::verify_package(&package, version.as_deref(), &language, &registry, trust_threshold) {
        Ok(result) => {
            result.display();
            
            if !result.is_trusted && !yes {
                println!("\npackage verification failed. use --yes to force install anyway.");
                return;
            }
            
            if yes {
                println!("auto-approve mode enabled");
            }
            if !extra_args.is_empty() {
                println!("extra args: {:?}", extra_args);
            }
            
            // todo: invoke actual package manager
            println!("would now invoke package manager for installation");
        }
        Err(e) => {
            println!("verification error: {}", e);
        }
    }
}

fn handle_verify(package: String, version: Option<String>, lang: Option<Language>) {
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
            println!("could not detect language - please specify with --lang flag");
            println!("supported languages: python, rust, go");
            return;
        }
    };
    
    // load registry and verify package
    let registry = match load_registry() {
        Ok(reg) => reg,
        Err(e) => {
            println!("failed to load registry: {}", e);
            return;
        }
    };
    
    let trust_threshold = 8.0; // todo: load from config
    match verify::verify_package(&package, version.as_deref(), &language, &registry, trust_threshold) {
        Ok(result) => {
            result.display();
        }
        Err(e) => {
            println!("verification error: {}", e);
        }
    }
}
