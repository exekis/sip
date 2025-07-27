use jsonschema::JSONSchema;
use crate::cli::Language;
use crate::sip::package::PackageRecord;
use std::fs;
use std::path::Path;

// embed registry data at compile time - makes binary portable
const SCHEMA_JSON: &str = include_str!("../../registry/schema/sip-package.json");
const PYTHON_PACKAGES_JSON: &str = include_str!("../../registry/data/python/trusted-packages.json");
const RUST_CRATES_JSON: &str = include_str!("../../registry/data/rust/trusted-crates.json");
const GO_MODULES_JSON: &str = include_str!("../../registry/data/go/trusted-modules.json");

#[derive(Debug)]
pub struct Registry {
    pub python_packages: Vec<PackageRecord>,
    pub rust_crates: Vec<PackageRecord>,
    pub go_modules: Vec<PackageRecord>,
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
    
    // load registry with ability to modify and save back to disk
    pub fn load_mutable() -> Result<Self, Box<dyn std::error::Error>> {
        let schema = Self::load_embedded_schema()?;
        
        // try to load from local files first, fall back to embedded
        let python_packages = Self::load_from_file_or_embedded(
            "registry/data/python/trusted-packages.json",
            PYTHON_PACKAGES_JSON,
            &schema
        )?;
        
        let rust_crates = Self::load_from_file_or_embedded(
            "registry/data/rust/trusted-crates.json", 
            RUST_CRATES_JSON,
            &schema
        )?;
        
        let go_modules = Self::load_from_file_or_embedded(
            "registry/data/go/trusted-modules.json",
            GO_MODULES_JSON, 
            &schema
        )?;
        
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
    
    fn load_embedded_packages(json_str: &str, schema: &JSONSchema) -> Result<Vec<PackageRecord>, Box<dyn std::error::Error>> {
        let packages_json: serde_json::Value = serde_json::from_str(json_str)?;
        
        // validate against schema
        if let Err(errors) = schema.validate(&packages_json) {
            let error_msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(format!("embedded registry schema validation failed: {}", error_msgs.join(", ")).into());
        }
        
        let packages: Vec<PackageRecord> = serde_json::from_str(json_str)?;
        Ok(packages)
    }
    
    fn load_from_file_or_embedded(
        file_path: &str,
        embedded_data: &str,
        schema: &JSONSchema
    ) -> Result<Vec<PackageRecord>, Box<dyn std::error::Error>> {
        if Path::new(file_path).exists() {
            println!("loading from file: {}", file_path);
            let content = fs::read_to_string(file_path)?;
            let packages_json: serde_json::Value = serde_json::from_str(&content)?;
            
            // validate against schema
            if let Err(errors) = schema.validate(&packages_json) {
                let error_msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
                return Err(format!("file registry schema validation failed: {}", error_msgs.join(", ")).into());
            }
            
            let packages: Vec<PackageRecord> = serde_json::from_str(&content)?;
            Ok(packages)
        } else {
            println!("file {} not found, using embedded data", file_path);
            Self::load_embedded_packages(embedded_data, schema)
        }
    }
    
    pub fn lookup_package(&self, name: &str, lang: &Language) -> Option<&PackageRecord> {
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
    
    pub fn add_package(&mut self, package: PackageRecord, lang: &Language) {
        // remove existing entry if present
        self.remove_package(&package.name, lang);
        
        // add new entry
        match lang {
            Language::Python => self.python_packages.push(package),
            Language::Rust => self.rust_crates.push(package),
            Language::Go => self.go_modules.push(package),
        }
    }
    
    pub fn remove_package(&mut self, name: &str, lang: &Language) -> bool {
        match lang {
            Language::Python => {
                if let Some(pos) = self.python_packages.iter().position(|pkg| pkg.name == name) {
                    self.python_packages.remove(pos);
                    true
                } else {
                    false
                }
            }
            Language::Rust => {
                if let Some(pos) = self.rust_crates.iter().position(|pkg| pkg.name == name) {
                    self.rust_crates.remove(pos);
                    true
                } else {
                    false
                }
            }
            Language::Go => {
                if let Some(pos) = self.go_modules.iter().position(|pkg| pkg.name == name) {
                    self.go_modules.remove(pos);
                    true
                } else {
                    false
                }
            }
        }
    }
    
    pub fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        // ensure registry directory exists
        fs::create_dir_all("registry/data/python")?;
        fs::create_dir_all("registry/data/rust")?;
        fs::create_dir_all("registry/data/go")?;
        
        // save each language registry
        let python_json = serde_json::to_string_pretty(&self.python_packages)?;
        fs::write("registry/data/python/trusted-packages.json", python_json)?;
        
        let rust_json = serde_json::to_string_pretty(&self.rust_crates)?;
        fs::write("registry/data/rust/trusted-crates.json", rust_json)?;
        
        let go_json = serde_json::to_string_pretty(&self.go_modules)?;
        fs::write("registry/data/go/trusted-modules.json", go_json)?;
        
        println!("registry saved to disk");
        Ok(())
    }
    
    pub fn list_packages(&self, lang: Option<&Language>) -> Vec<&PackageRecord> {
        match lang {
            Some(Language::Python) => self.python_packages.iter().collect(),
            Some(Language::Rust) => self.rust_crates.iter().collect(),
            Some(Language::Go) => self.go_modules.iter().collect(),
            None => {
                let mut all_packages = Vec::new();
                all_packages.extend(self.python_packages.iter());
                all_packages.extend(self.rust_crates.iter());
                all_packages.extend(self.go_modules.iter());
                all_packages
            }
        }
    }
}
