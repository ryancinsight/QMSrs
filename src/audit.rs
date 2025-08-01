use crate::error::{QmsError, Result};
use crate::database::Database;
use crate::config::ComplianceConfig;
use crate::logging::{AuditLogEntry, AuditOutcome};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit trail manager for FDA compliance
pub struct AuditManager {
    database: Database,
}

impl AuditManager {
    /// Create a new audit manager with database connection
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Log an action for audit trail
    pub fn log_action(
        &self,
        user_id: &str,
        action: &str,
        resource: &str,
        outcome: &str,
        metadata: Option<String>,
    ) -> Result<()> {
        use uuid::Uuid;
        use chrono::Utc;

        let audit_outcome = match outcome.to_lowercase().as_str() {
            "success" => AuditOutcome::Success,
            "failure" => AuditOutcome::Failure,
            "warning" => AuditOutcome::Warning,
            _ => AuditOutcome::Success, // Default to success
        };

        let metadata_json = metadata
            .map(|m| serde_json::from_str(&m).unwrap_or_else(|_| serde_json::Value::String(m)))
            .unwrap_or(serde_json::Value::Null);

        let entry = AuditLogEntry {
            timestamp: Utc::now(),
            user_id: user_id.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            outcome: audit_outcome,
            ip_address: Some("127.0.0.1".to_string()), // Default for now
            session_id: Uuid::new_v4().to_string(),
            metadata: metadata_json,
            compliance_version: "21CFR820".to_string(),
            signature_hash: None,
        };

        // Store the audit entry in the database
        self.database.insert_audit_entry(&entry)?;
        Ok(())
    }

    /// Log an audit event
    pub fn log_event(&mut self, entry: AuditLogEntry) -> Result<()> {
        entry.validate()?;
        self.database.insert_audit_entry(&entry)?;
        Ok(())
    }

    /// Generate FDA compliance report
    pub fn generate_compliance_report(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<ComplianceReport> {
        let integrity_report = self.database.verify_audit_integrity()?;
        
        Ok(ComplianceReport {
            report_id: uuid::Uuid::new_v4().to_string(),
            generated_at: Utc::now(),
            period_start: start_date,
            period_end: end_date,
            total_audit_entries: integrity_report.total_entries,
            integrity_verified: integrity_report.integrity_verified,
            compliance_status: if integrity_report.integrity_verified {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::NonCompliant
            },
            details: integrity_report.details,
        })
    }
}

/// FDA compliance report structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub report_id: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_audit_entries: u64,
    pub integrity_verified: bool,
    pub compliance_status: ComplianceStatus,
    pub details: String,
}

/// Compliance status enumeration
#[derive(Debug, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    Warning,
}

/// Simple audit logger for module-level audit logging
pub struct AuditLogger {
    session_id: String,
}

impl AuditLogger {
    /// Create new audit logger with session ID
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }

    /// Create a test audit logger for unit tests
    pub fn new_test() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
        }
    }

    /// Log an audit event
    pub async fn log_event(
        &self,
        user_id: &str,
        action: &str,
        resource: &str,
        outcome: &str,
        details: Option<String>,
    ) -> Result<()> {
        let audit_outcome = match outcome {
            "SUCCESS" => AuditOutcome::Success,
            "FAILURE" => AuditOutcome::Failure,
            "WARNING" => AuditOutcome::Warning,
            other => return Err(QmsError::Validation {
                field: "outcome".to_string(),
                message: format!("Invalid audit outcome string: '{}'", other),
            }),
        };

        let mut entry = AuditLogEntry::new(
            user_id.to_string(),
            action.to_string(),
            resource.to_string(),
            audit_outcome,
            self.session_id.clone(),
        );

        if let Some(details) = details {
            let metadata = serde_json::json!({
                "details": details
            });
            entry = entry.with_metadata(metadata);
        }

        // Validate and log the entry
        entry.validate()?;
        entry.log();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    #[test]
    fn test_audit_manager_creation() {
        use crate::config::DatabaseConfig;
        
        let config = DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false,
            backup_interval_hours: 24,
            backup_retention_days: 90,
        };

        let database = Database::new(config).unwrap();
        let _audit_manager = AuditManager::new(database);
    }

    #[test]
    fn test_compliance_report_generation() {
        let config = DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false,
            backup_interval_hours: 24,
            backup_retention_days: 90,
        };

        let database = Database::new(config).unwrap();
        let audit_manager = AuditManager::new(database);
        
        let start = Utc::now() - chrono::Duration::days(30);
        let end = Utc::now();
        
        let report = audit_manager.generate_compliance_report(start, end).unwrap();
        assert!(!report.report_id.is_empty());
    }
}