use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::{Result, QmsError};

/// Main configuration structure for QMS system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application-specific settings
    pub application: ApplicationConfig,
    
    /// FDA compliance settings
    pub compliance: ComplianceConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Organization name for FDA reporting
    pub organization_name: String,
    
    /// FDA registration number
    pub fda_registration: Option<String>,
    
    /// ISO 13485 certificate number
    pub iso_certificate: Option<String>,
    
    /// Application data directory
    #[serde(default = "default_data_dir")]
    pub data_directory: String,
}

/// FDA compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Enable strict FDA validation mode
    #[serde(default = "default_true")]
    pub strict_validation: bool,
    
    /// Audit retention period in days (minimum 7 years for FDA)
    #[serde(default = "default_audit_retention")]
    pub audit_retention_days: u32,
    
    /// Require electronic signatures for critical operations
    #[serde(default = "default_true")]
    pub require_electronic_signatures: bool,
    
    /// CFR Part 11 compliance mode
    #[serde(default = "default_true")]
    pub cfr_part_11_mode: bool,
}

/// Logging configuration for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub level: String,
    
    /// Log file path
    #[serde(default = "default_log_file")]
    pub file: String,
    
    /// Use JSON format for structured logging
    #[serde(default = "default_true")]
    pub json_format: bool,
    
    /// Maximum log file size in MB
    #[serde(default = "default_log_size")]
    pub max_size_mb: u64,
    
    /// Number of log files to retain
    #[serde(default = "default_log_retention")]
    pub retention_count: u32,
    
    /// Encrypt log files for FDA compliance
    #[serde(default = "default_true")]
    pub encrypt_logs: bool,
}

impl Config {
    /// Load configuration from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| QmsError::Config {
                message: format!("Failed to read config file: {}", e),
            })?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| QmsError::Config {
                message: format!("Failed to parse config file: {}", e),
            })?;

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration for FDA compliance
    pub fn validate(&self) -> Result<()> {
        // Validate audit retention meets FDA requirements (minimum 7 years)
        if self.compliance.audit_retention_days < 2555 {
            return Err(QmsError::Validation {
                field: "audit_retention_days".to_string(),
                message: "FDA requires minimum 7 years (2555 days) audit retention".to_string(),
            });
        }

        // Validate organization name is provided
        if self.application.organization_name.trim().is_empty() {
            return Err(QmsError::Validation {
                field: "organization_name".to_string(),
                message: "Organization name is required for FDA compliance".to_string(),
            });
        }

        Ok(())
    }

    /// Generate sample configuration
    pub fn generate_sample() -> String {
        toml::to_string_pretty(&Self::default()).unwrap_or_else(|_| String::new())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            application: ApplicationConfig::default(),
            compliance: ComplianceConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            organization_name: "Medical Device Company".to_string(),
            fda_registration: None,
            iso_certificate: None,
            data_directory: default_data_dir(),
        }
    }
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            strict_validation: default_true(),
            audit_retention_days: default_audit_retention(),
            require_electronic_signatures: default_true(),
            cfr_part_11_mode: default_true(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: default_log_file(),
            json_format: default_true(),
            max_size_mb: default_log_size(),
            retention_count: default_log_retention(),
            encrypt_logs: default_true(),
        }
    }
}

// Default value functions
fn default_true() -> bool { true }
fn default_data_dir() -> String { "./qms-data".to_string() }
fn default_audit_retention() -> u32 { 2555 } // 7 years
fn default_log_level() -> String { "info".to_string() }
fn default_log_file() -> String { "./qms-data/audit.log".to_string() }
fn default_log_size() -> u64 { 10 }
fn default_log_retention() -> u32 { 30 }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_validation_success() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_audit_retention() {
        let mut config = Config::default();
        config.compliance.audit_retention_days = 365; // Less than 7 years
        
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_validation_organization_name() {
        let mut config = Config::default();
        config.application.organization_name = "".to_string();
        
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_sample_generation() {
        let sample = Config::generate_sample();
        assert!(!sample.is_empty());
        assert!(sample.contains("organization_name"));
        assert!(sample.contains("audit_retention_days"));
    }

    #[test]
    fn test_default_values_compliance() {
        let config = Config::default();
        
        // Test FDA compliance defaults
        assert!(config.compliance.strict_validation);
        assert!(config.compliance.cfr_part_11_mode);
        assert!(config.compliance.require_electronic_signatures);
        assert_eq!(config.compliance.audit_retention_days, 2555); // 7 years
    }
}