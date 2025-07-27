use crate::cli::Language;
use crate::sip::registry::{Registry, PackageEntry};

#[derive(Debug)]
pub struct VerificationResult {
    pub package_name: String,
    pub package_entry: Option<PackageEntry>,
    pub trust_score: Option<f64>,
    pub is_trusted: bool,
    pub trust_threshold: f64,
}

impl VerificationResult {
    pub fn display(&self) {
        println!("package: {}", self.package_name);
        
        match &self.package_entry {
            Some(entry) => {
                println!("version: {}", entry.version);
                println!("trust score: {:.1}/10.0", entry.trust_score);
                println!("last reviewed: {}", entry.last_reviewed);
                println!("endorsed by: {}", entry.endorsed_by.join(", "));
                
                if let Some(source) = &entry.source {
                    println!("source: {}", source);
                }
                
                if self.is_trusted {
                    println!("✓ trusted (meets threshold of {:.1})", self.trust_threshold);
                } else {
                    println!("⚠ below trust threshold (requires {:.1})", self.trust_threshold);
                }
            }
            None => {
                println!("✘ not found in trusted registry");
                println!("this package has not been reviewed or endorsed");
            }
        }
    }
}

pub fn verify_package(
    package: &str, 
    _version: Option<&str>, 
    lang: &Language,
    registry: &Registry,
    trust_threshold: f64,
) -> Result<VerificationResult, Box<dyn std::error::Error>> {
    let package_entry = registry.lookup_package(package, lang);
    
    let (trust_score, is_trusted) = match &package_entry {
        Some(entry) => {
            let score = entry.trust_score;
            (Some(score), score >= trust_threshold)
        }
        None => (None, false),
    };
    
    Ok(VerificationResult {
        package_name: package.to_string(),
        package_entry: package_entry.cloned(),
        trust_score,
        is_trusted,
        trust_threshold,
    })
}
