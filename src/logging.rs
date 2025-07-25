use crate::{Result, QmsError, config::LoggingConfig};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::{rolling, non_blocking};
use std::path::Path;

/// Initialize FDA-compliant audit trail logging
pub fn init_tracing(config: &LoggingConfig) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    // Create log directory if it doesn't exist
    let log_path = Path::new(&config.file);
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| QmsError::FileSystem {
                path: parent.display().to_string(),
                message: format!("Failed to create log directory: {}", e),
            })?;
    }

    // Set up rolling file appender for audit logs
    let fallback_dir = Path::new("/var/log/qms");
    if !fallback_dir.exists() {
        std::fs::create_dir_all(fallback_dir).map_err(|e| QmsError::FileSystem {
            path: fallback_dir.display().to_string(),
            message: format!("Failed to create fallback log directory: {}", e),
        })?;
    }
    let file_appender = rolling::daily(
        log_path.parent().unwrap_or(fallback_dir),
        log_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("qms-audit.log")
    );

    let (non_blocking, guard) = non_blocking(file_appender);

    // Configure the environment filter
    let env_filter = EnvFilter::try_new(&config.level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Set up the subscriber with both console and file outputs
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_file(true)
        );

    // Add file logging layer
    let subscriber = if config.json_format {
        subscriber.with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking)
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_file(true)
        )
    } else {
        subscriber.with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_file(true)
        )
    };

    subscriber.init();

    // Log initialization
    tracing::info!(
        component = "audit_trail",
        action = "logging_initialized",
        config = ?config,
        "FDA-compliant audit trail logging initialized"
    );

    Ok(guard)
}

/// Audit log entry structure for FDA compliance
#[derive(Debug, serde::Serialize)]
pub struct AuditLogEntry {
    /// RFC 3339 timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// User ID performing the action
    pub user_id: String,
    
    /// Action performed
    pub action: String,
    
    /// Resource affected
    pub resource: String,
    
    /// Outcome of the action (success/failure)
    pub outcome: AuditOutcome,
    
    /// IP address (if applicable)
    pub ip_address: Option<String>,
    
    /// Session ID
    pub session_id: String,
    
    /// Additional metadata
    pub metadata: serde_json::Value,
    
    /// FDA compliance version
    pub compliance_version: String,
    
    /// Digital signature hash
    pub signature_hash: Option<String>,
}

/// Audit outcome enumeration
#[derive(Debug, serde::Serialize, Clone, Copy)]
pub enum AuditOutcome {
    Success,
    Failure,
    Warning,
}

impl AuditOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditOutcome::Success => "SUCCESS",
            AuditOutcome::Failure => "FAILURE",
            AuditOutcome::Warning => "WARNING",
        }
    }
}

impl AuditLogEntry {
    /// Create a new audit log entry
    pub fn new(
        user_id: String,
        action: String,
        resource: String,
        outcome: AuditOutcome,
        session_id: String,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            user_id,
            action,
            resource,
            outcome,
            ip_address: None,
            session_id,
            metadata: serde_json::Value::Null,
            compliance_version: crate::FDA_CFR_PART_820_VERSION.to_string(),
            signature_hash: None,
        }
    }

    /// Add IP address to audit entry
    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Add metadata to audit entry
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add digital signature to audit entry
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature_hash = Some(signature);
        self
    }

    /// Log this entry using tracing
    pub fn log(&self) {
        tracing::info!(
            audit_entry = true,
            timestamp = %self.timestamp,
            user_id = %self.user_id,
            action = %self.action,
            resource = %self.resource,
            outcome = %self.outcome.as_str(),
            ip_address = ?self.ip_address,
            session_id = %self.session_id,
            metadata = %self.metadata,
            compliance_version = %self.compliance_version,
            signature_hash = ?self.signature_hash,
            "FDA audit trail entry"
        );
    }

    /// Validate audit entry completeness for FDA compliance
    pub fn validate(&self) -> Result<()> {
        if self.user_id.is_empty() {
            return Err(QmsError::AuditTrail {
                message: "User ID is required for FDA audit trail".to_string(),
            });
        }

        if self.action.is_empty() {
            return Err(QmsError::AuditTrail {
                message: "Action is required for FDA audit trail".to_string(),
            });
        }

        if self.resource.is_empty() {
            return Err(QmsError::AuditTrail {
                message: "Resource is required for FDA audit trail".to_string(),
            });
        }

        if self.session_id.is_empty() {
            return Err(QmsError::AuditTrail {
                message: "Session ID is required for FDA audit trail".to_string(),
            });
        }

        Ok(())
    }
}

/// Macro for creating audit log entries
#[macro_export]
macro_rules! audit_log {
    ($user_id:expr, $action:expr, $resource:expr, $outcome:expr, $session_id:expr) => {
        {
            let entry = $crate::logging::AuditLogEntry::new(
                $user_id.to_string(),
                $action.to_string(),
                $resource.to_string(),
                $outcome,
                $session_id.to_string(),
            );
            entry.log();
            entry
        }
    };
    
    ($user_id:expr, $action:expr, $resource:expr, $outcome:expr, $session_id:expr, $metadata:expr) => {
        {
            let entry = $crate::logging::AuditLogEntry::new(
                $user_id.to_string(),
                $action.to_string(),
                $resource.to_string(),
                $outcome,
                $session_id.to_string(),
            ).with_metadata($metadata);
            entry.log();
            entry
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_log_entry_creation() {
        let entry = AuditLogEntry::new(
            "user123".to_string(),
            "login".to_string(),
            "authentication_system".to_string(),
            AuditOutcome::Success,
            "session456".to_string(),
        );

        assert_eq!(entry.user_id, "user123");
        assert_eq!(entry.action, "login");
        assert_eq!(entry.resource, "authentication_system");
        assert_eq!(entry.outcome.as_str(), "SUCCESS");
        assert_eq!(entry.session_id, "session456");
    }

    #[test]
    fn test_audit_log_entry_validation() {
        let mut entry = AuditLogEntry::new(
            "user123".to_string(),
            "login".to_string(),
            "auth".to_string(),
            AuditOutcome::Success,
            "session456".to_string(),
        );

        // Valid entry should pass
        assert!(entry.validate().is_ok());

        // Empty user_id should fail
        entry.user_id = String::new();
        assert!(entry.validate().is_err());
    }

    #[test]
    fn test_audit_outcome_as_str() {
        assert_eq!(AuditOutcome::Success.as_str(), "SUCCESS");
        assert_eq!(AuditOutcome::Failure.as_str(), "FAILURE");
        assert_eq!(AuditOutcome::Warning.as_str(), "WARNING");
    }

    #[test]
    fn test_audit_log_entry_with_metadata() {
        let metadata = serde_json::json!({
            "additional_info": "test data",
            "severity": "high"
        });

        let entry = AuditLogEntry::new(
            "user123".to_string(),
            "document_update".to_string(),
            "SOP-001".to_string(),
            AuditOutcome::Success,
            "session456".to_string(),
        ).with_metadata(metadata.clone());

        assert_eq!(entry.metadata, metadata);
    }

    #[test]
    fn test_logging_config_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test-audit.log");

        let config = LoggingConfig {
            level: "info".to_string(),
            file: log_file.display().to_string(),
            json_format: true,
            max_size_mb: 10,
            retention_count: 5,
            encrypt_logs: true,
        };

        let result = init_tracing(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_log_macro() {
        let entry = audit_log!(
            "user123",
            "test_action",
            "test_resource",
            AuditOutcome::Success,
            "session456"
        );

        assert_eq!(entry.user_id, "user123");
        assert_eq!(entry.action, "test_action");
    }
}