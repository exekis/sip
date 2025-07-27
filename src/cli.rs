use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sip")]
#[command(about = "safe install proxy - a trusted registry wrapper for package managers")]
#[command(long_about = "sip is a drop-in cli wrapper for native package managers (pip, cargo, go) that enforces trusted registry checks before installation")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// install a package after verifying against trusted registry
    Install {
        /// name of the package to install
        #[arg(value_name = "PACKAGE")]
        package: String,

        /// version constraint (optional)
        #[arg(short, long)]
        version: Option<String>,

        /// explicitly specify the language/ecosystem
        #[arg(short, long, value_enum)]
        lang: Option<Language>,

        /// skip interactive prompts and fail on unverified packages
        #[arg(short, long)]
        yes: bool,

        /// pass additional arguments to the underlying package manager
        #[arg(last = true)]
        extra_args: Vec<String>,
    },

    /// verify a package against the trusted registry without installing
    Verify {
        /// name of the package to verify
        #[arg(value_name = "PACKAGE")]
        package: String,

        /// specific version to verify (optional)
        #[arg(short, long)]
        version: Option<String>,

        /// explicitly specify the language/ecosystem
        #[arg(short, long, value_enum)]
        lang: Option<Language>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Language {
    Python,
    Rust,
    Go,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Python => write!(f, "python"),
            Language::Rust => write!(f, "rust"),
            Language::Go => write!(f, "go"),
        }
    }
}

pub fn parse() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_install_command() {
        let cli = Cli::try_parse_from(vec!["sip", "install", "requests"]).unwrap();
        match cli.command {
            Commands::Install { package, .. } => {
                assert_eq!(package, "requests");
            }
            _ => panic!("expected install command"),
        }
    }

    #[test]
    fn test_install_with_version() {
        let cli = Cli::try_parse_from(vec![
            "sip", "install", "requests", "--version", "2.31.0",
        ]).unwrap();
        match cli.command {
            Commands::Install { package, version, .. } => {
                assert_eq!(package, "requests");
                assert_eq!(version, Some("2.31.0".to_string()));
            }
            _ => panic!("expected install command"),
        }
    }

    #[test]
    fn test_install_with_language() {
        let cli = Cli::try_parse_from(vec![
            "sip", "install", "tokio", "--lang", "rust",
        ]).unwrap();
        match cli.command {
            Commands::Install { package, lang, .. } => {
                assert_eq!(package, "tokio");
                assert!(matches!(lang, Some(Language::Rust)));
            }
            _ => panic!("expected install command"),
        }
    }

    #[test]
    fn test_verify_command() {
        let cli = Cli::try_parse_from(vec!["sip", "verify", "numpy"]).unwrap();
        match cli.command {
            Commands::Verify { package, .. } => {
                assert_eq!(package, "numpy");
            }
            _ => panic!("expected verify command"),
        }
    }

    #[test]
    fn test_install_with_extra_args() {
        let cli = Cli::try_parse_from(vec![
            "sip", "install", "requests", "--", "--user", "--upgrade",
        ]).unwrap();
        match cli.command {
            Commands::Install { package, extra_args, .. } => {
                assert_eq!(package, "requests");
                assert_eq!(extra_args, vec!["--user", "--upgrade"]);
            }
            _ => panic!("expected install command"),
        }
    }
}
