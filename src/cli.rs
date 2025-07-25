use clap::Parser;
use std::path::PathBuf;

/// FDA Compliant Medical Device Quality Management System
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "qmsrs")]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "qms-config.toml")]
    pub config_path: PathBuf,

    /// Database URL override
    #[arg(short, long)]
    pub database_url: Option<String>,

    /// Log level override (trace, debug, info, warn, error)
    #[arg(short, long)]
    pub log_level: Option<String>,

    /// Enable development mode (less strict validation)
    #[arg(long)]
    pub dev_mode: bool,

    /// Enable audit trail verification on startup
    #[arg(long, default_value = "true")]
    pub verify_audit_trail: bool,

    /// Initialize database schema and exit
    #[arg(long)]
    pub init_db: bool,

    /// Run in headless mode (no TUI)
    #[arg(long)]
    pub headless: bool,

    /// Generate sample configuration file and exit
    #[arg(long)]
    pub generate_config: bool,
}

impl Cli {
    /// Validate CLI arguments for FDA compliance
    pub fn validate(&self) -> crate::Result<()> {
        // Ensure audit trail verification is enabled in production
        if !self.dev_mode && !self.verify_audit_trail {
            return Err(crate::QmsError::Validation {
                field: "verify_audit_trail".to_string(),
                message: "Audit trail verification must be enabled in production mode for FDA compliance".to_string(),
            });
        }

        // Validate config file path
        if !self.generate_config && !self.config_path.exists() && !self.init_db {
            return Err(crate::QmsError::Config {
                message: format!("Configuration file not found: {}", self.config_path.display()),
            });
        }

        Ok(())
    }

    /// Get effective log level
    pub fn effective_log_level(&self) -> String {
        self.log_level.clone().unwrap_or_else(|| {
            if self.dev_mode {
                "debug".to_string()
            } else {
                "info".to_string()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cli_default_values() {
        let cli = Cli::parse_from(&["qmsrs"]);
        assert_eq!(cli.config_path, PathBuf::from("qms-config.toml"));
        assert_eq!(cli.database_url, None);
        assert_eq!(cli.log_level, None);
        assert!(!cli.dev_mode);
        assert!(cli.verify_audit_trail);
        assert!(!cli.init_db);
        assert!(!cli.headless);
        assert!(!cli.generate_config);
    }

    #[test]
    fn test_cli_validation_production_mode() {
        let mut cli = Cli::parse_from(&["qmsrs"]);
        cli.verify_audit_trail = false;
        cli.dev_mode = false;
        
        let result = cli.validate();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            crate::QmsError::Validation { field, message } => {
                assert_eq!(field, "verify_audit_trail");
                assert!(message.contains("FDA compliance"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_cli_validation_dev_mode() {
        let mut cli = Cli::parse_from(&["qmsrs"]);
        cli.verify_audit_trail = false;
        cli.dev_mode = true;
        cli.generate_config = true; // Skip config file check
        
        let result = cli.validate();
        assert!(result.is_ok()); // Should be ok in dev mode
    }

    #[test]
    fn test_effective_log_level() {
        let mut cli = Cli::parse_from(&["qmsrs"]);
        
        // Test default production level
        assert_eq!(cli.effective_log_level(), "info");
        
        // Test dev mode default
        cli.dev_mode = true;
        assert_eq!(cli.effective_log_level(), "debug");
        
        // Test explicit override
        cli.log_level = Some("trace".to_string());
        assert_eq!(cli.effective_log_level(), "trace");
    }

    #[test]
    fn test_cli_parsing_with_args() {
        let cli = Cli::parse_from(&[
            "qmsrs",
            "--config-path", "/tmp/test.toml",
            "--database-url", "sqlite://test.db",
            "--log-level", "debug",
            "--dev-mode",
            "--headless",
        ]);

        assert_eq!(cli.config_path, PathBuf::from("/tmp/test.toml"));
        assert_eq!(cli.database_url, Some("sqlite://test.db".to_string()));
        assert_eq!(cli.log_level, Some("debug".to_string()));
        assert!(cli.dev_mode);
        assert!(cli.headless);
    }
}