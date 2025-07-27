use serde::{Deserialize, Serialize};
use jsonschema::JSONSchema;
use crate::cli::Language;

// embed registry data at compile time - makes binary portable
const SCHEMA_JSON: &str = include_str!("../../registry/schema/sip-package.json");
const PYTHON_PACKAGES_JSON: &str = include_str!("../../registry/data/python/trusted-packages.json");
const RUST_CRATES_JSON: &str = include_str!("../../registry/data/rust/trusted-crates.json");
const GO_MODULES_JSON: &str = include_str!("../../registry/data/go/trusted-modules.json");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub hash: String,
    pub trust_score: f64,
    pub endorsed_by: Vec<String>,
    pub last_reviewed: String,
    pub source: Option<String>,
}

#[derive(Debug)]
pub struct Registry {
    pub python_packages: Vec<PackageEntry>,
    pub rust_crates: Vec<PackageEntry>,
    pub go_modules: Vec<PackageEntry>,
    schema: JSONSchema,
}

impl Registry {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let schema = Self::load_embedded_schema()?;
        
        let python_packages = Self::load_embedded_packages(PYTHON_PACKAGES_JSON, &schema)?;
        let rust_crates = Self::load_embedded_packages(RUST_CRATES_JSON, &schema)?;
        let go_modules = Self::load_embedded_packages(GO_MODULES_JSON, &schema)?;
        
        println!("loaded {} python packages", python_packages.len());
        println!("loaded {} rust crates", rust_crates.len());
        println!("loaded {} go modules", go_modules.len());
        
        Ok(Registry {
            python_packages,
            rust_crates,
            go_modules,
            schema,
        })
    }
    
    // kept for backwards compatibility but now just calls new()
    pub fn load_from_directory(_registry_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new()
    }
    
    fn load_embedded_schema() -> Result<JSONSchema, Box<dyn std::error::Error>> {
        let schema_json: serde_json::Value = serde_json::from_str(SCHEMA_JSON)?;
        let compiled = JSONSchema::compile(&schema_json)
            .map_err(|e| format!("failed to compile embedded json schema: {}", e))?;
        Ok(compiled)
    }
    
    fn load_embedded_packages(json_str: &str, schema: &JSONSchema) -> Result<Vec<PackageEntry>, Box<dyn std::error::Error>> {
        let packages_json: serde_json::Value = serde_json::from_str(json_str)?;
        
        // validate against schema
        if let Err(errors) = schema.validate(&packages_json) {
            let error_msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(format!("embedded registry schema validation failed: {}", error_msgs.join(", ")).into());
        }
        
        let packages: Vec<PackageEntry> = serde_json::from_str(json_str)?;
        Ok(packages)
    }
    
    pub fn lookup_package(&self, name: &str, lang: &Language) -> Option<&PackageEntry> {
        match lang {
            Language::Python => {
                self.python_packages.iter().find(|pkg| pkg.name == name)
            }
            Language::Rust => {
                self.rust_crates.iter().find(|pkg| pkg.name == name)
            }
            Language::Go => {
                self.go_modules.iter().find(|pkg| pkg.name == name)
            }
        }
    }
}
