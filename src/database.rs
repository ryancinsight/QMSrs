use crate::{Result, QmsError, logging::AuditLogEntry, config::DatabaseConfig};
use rusqlite::{Connection, params};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Database manager for FDA-compliant QMS with connection pooling
#[derive(Clone)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Create new database connection with connection pool
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        // Ensure database directory exists for file-based databases
        if config.url != ":memory:" {
            if let Some(parent) = Path::new(&config.url).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| QmsError::FileSystem {
                        path: parent.display().to_string(),
                        message: format!("Failed to create database directory: {}", e),
                    })?;
            }
        }

        // Create connection manager
        let manager = SqliteConnectionManager::file(&config.url)
            .with_init(move |conn| {
                // Configure pragma settings for FDA compliance
                if config.wal_mode {
                    conn.execute_batch("PRAGMA journal_mode=WAL")?;
                }
                conn.execute_batch("PRAGMA foreign_keys=ON")?;
                conn.execute_batch("PRAGMA synchronous=FULL")?;
                conn.execute_batch("PRAGMA secure_delete=ON")?;
                Ok(())
            });

        // Create connection pool
        let pool = Pool::builder()
            .max_size(config.max_connections)
            .build(manager)
            .map_err(|e| QmsError::Database {
                message: format!("Failed to create connection pool: {}", e),
            })?;

        let db = Self { pool };
        
        // Initialize schema using a connection from the pool
        db.initialize_schema()?;
        
        Ok(db)
    }

    /// Initialize database schema for FDA compliance
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| QmsError::Database {
                message: format!("Failed to get database connection: {}", e),
            })?;

        // Create audit trail table (critical for FDA compliance)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_trail (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                user_id TEXT NOT NULL,
                action TEXT NOT NULL,
                resource TEXT NOT NULL,
                outcome TEXT NOT NULL,
                ip_address TEXT,
                session_id TEXT NOT NULL,
                metadata TEXT,
                compliance_version TEXT NOT NULL,
                signature_hash TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create users table with role-based access control
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                role TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                last_login TEXT,
                failed_login_attempts INTEGER NOT NULL DEFAULT 0,
                locked_until TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // TASK-017: CAPA System Database Schema
        // Create CAPA records table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS capa_records (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                capa_type TEXT NOT NULL CHECK (capa_type IN ('Corrective', 'Preventive', 'Combined')),
                priority TEXT NOT NULL CHECK (priority IN ('Critical', 'High', 'Medium', 'Low')),
                status TEXT NOT NULL CHECK (status IN ('Identified', 'InvestigationInProgress', 'RootCauseAnalysis', 'CorrectiveActionInProgress', 'PreventiveActionInProgress', 'EffectivenessVerification', 'Closed', 'Cancelled')),
                initiator_id TEXT NOT NULL,
                assigned_to TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                due_date TEXT,
                closed_date TEXT,
                source_document TEXT,
                related_risk_id TEXT,
                investigation_summary TEXT,
                root_cause TEXT,
                metadata TEXT, -- JSON blob for additional metadata
                FOREIGN KEY (initiator_id) REFERENCES users(id),
                FOREIGN KEY (assigned_to) REFERENCES users(id)
            )",
            [],
        )?;

        // Create CAPA actions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS capa_actions (
                id TEXT PRIMARY KEY,
                capa_id TEXT NOT NULL,
                action_type TEXT NOT NULL CHECK (action_type IN ('Corrective', 'Preventive')),
                description TEXT NOT NULL,
                assigned_to TEXT NOT NULL,
                due_date TEXT NOT NULL,
                completed_date TEXT,
                verification_method TEXT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('Planned', 'InProgress', 'Completed', 'Verified', 'Overdue')),
                evidence TEXT, -- JSON array of evidence file paths
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (capa_id) REFERENCES capa_records(id) ON DELETE CASCADE,
                FOREIGN KEY (assigned_to) REFERENCES users(id)
            )",
            [],
        )?;

        // Create CAPA effectiveness verification table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS capa_effectiveness_verification (
                id TEXT PRIMARY KEY,
                capa_id TEXT NOT NULL UNIQUE,
                verification_date TEXT NOT NULL,
                verifier_id TEXT NOT NULL,
                method TEXT NOT NULL,
                results TEXT NOT NULL,
                is_effective BOOLEAN NOT NULL,
                follow_up_required BOOLEAN NOT NULL,
                follow_up_actions TEXT, -- JSON array of follow-up actions
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (capa_id) REFERENCES capa_records(id) ON DELETE CASCADE,
                FOREIGN KEY (verifier_id) REFERENCES users(id)
            )",
            [],
        )?;

        // Create documents table for document control system
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                document_number TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                version TEXT NOT NULL,
                status TEXT NOT NULL,
                document_type TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                file_path TEXT,
                created_by TEXT NOT NULL,
                approved_by TEXT,
                effective_date TEXT,
                review_date TEXT,
                retirement_date TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (created_by) REFERENCES users(id),
                FOREIGN KEY (approved_by) REFERENCES users(id)
            )",
            [],
        )?;

        // Create document versions table for version control
        conn.execute(
            "CREATE TABLE IF NOT EXISTS document_versions (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                version TEXT NOT NULL,
                change_description TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                file_path TEXT,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (document_id) REFERENCES documents(id),
                FOREIGN KEY (created_by) REFERENCES users(id),
                UNIQUE(document_id, version)
            )",
            [],
        )?;

        // Create sessions table for session management
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                ip_address TEXT,
                user_agent TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_activity TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                expires_at TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )",
            [],
        )?;

        // Create risk assessments table for ISO 14971 compliance
        conn.execute(
            "CREATE TABLE IF NOT EXISTS risk_assessments (
                id TEXT PRIMARY KEY,
                device_name TEXT NOT NULL,
                hazard_description TEXT NOT NULL,
                hazardous_situation TEXT NOT NULL,
                foreseeable_sequence TEXT NOT NULL,
                harm_description TEXT NOT NULL,
                initial_severity INTEGER NOT NULL,
                initial_probability INTEGER NOT NULL,
                initial_risk_level INTEGER NOT NULL,
                acceptability TEXT NOT NULL,
                residual_severity INTEGER,
                residual_probability INTEGER,
                residual_risk_level INTEGER,
                residual_acceptability TEXT,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_by TEXT,
                updated_at TEXT,
                reviewed_by TEXT,
                reviewed_at TEXT,
                status TEXT NOT NULL DEFAULT 'Draft',
                FOREIGN KEY (created_by) REFERENCES users(id),
                FOREIGN KEY (updated_by) REFERENCES users(id),
                FOREIGN KEY (reviewed_by) REFERENCES users(id)
            )",
            [],
        )?;

        // Create control measures table for risk mitigation
        conn.execute(
            "CREATE TABLE IF NOT EXISTS control_measures (
                id TEXT PRIMARY KEY,
                risk_assessment_id TEXT NOT NULL,
                measure_type TEXT NOT NULL,
                description TEXT NOT NULL,
                implementation_details TEXT NOT NULL,
                effectiveness_verification TEXT NOT NULL,
                verification_status TEXT NOT NULL DEFAULT 'Pending',
                implemented_by TEXT NOT NULL,
                implemented_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                verified_by TEXT,
                verified_at TEXT,
                FOREIGN KEY (risk_assessment_id) REFERENCES risk_assessments(id),
                FOREIGN KEY (implemented_by) REFERENCES users(id),
                FOREIGN KEY (verified_by) REFERENCES users(id)
            )",
            [],
        )?;

        // Create indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_trail_timestamp ON audit_trail(timestamp)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_trail_user_id ON audit_trail(user_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_risk_assessments_status ON risk_assessments(status)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_risk_assessments_device ON risk_assessments(device_name)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_control_measures_risk_id ON control_measures(risk_assessment_id)",
            [],
        )?;

        Ok(())
    }

    /// Insert audit trail entry
    pub fn insert_audit_entry(&self, entry: &AuditLogEntry) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| QmsError::Database {
                message: format!("Failed to get database connection: {}", e),
            })?;

        let id = Uuid::new_v4().to_string();
        
        conn.execute(
            "INSERT INTO audit_trail (
                id, timestamp, user_id, action, resource, outcome,
                ip_address, session_id, metadata, compliance_version, signature_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id,
                entry.timestamp.to_rfc3339(),
                entry.user_id,
                entry.action,
                entry.resource,
                entry.outcome.as_str(),
                entry.ip_address,
                entry.session_id,
                serde_json::to_string(&entry.metadata)?,
                entry.compliance_version,
                entry.signature_hash
            ],
        )?;

        Ok(())
    }

    /// Get audit trail entries with pagination
    pub fn get_audit_entries(
        &self,
        limit: i64,
        offset: i64,
        user_id: Option<&str>,
    ) -> Result<Vec<AuditTrailEntry>> {
        let conn = self.pool.get()
            .map_err(|e| QmsError::Database {
                message: format!("Failed to get database connection: {}", e),
            })?;

        let mut query = "SELECT * FROM audit_trail".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(uid) = user_id {
            query.push_str(" WHERE user_id = ?");
            params.push(Box::new(uid.to_string()));
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let mut stmt = conn.prepare(&query)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let audit_iter = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(AuditTrailEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                user_id: row.get(2)?,
                action: row.get(3)?,
                resource: row.get(4)?,
                outcome: row.get(5)?,
                ip_address: row.get(6)?,
                session_id: row.get(7)?,
                metadata: row.get(8)?,
                compliance_version: row.get(9)?,
                signature_hash: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;

        let mut entries = Vec::new();
        for entry in audit_iter {
            entries.push(entry?);
        }

        Ok(entries)
    }

    /// Verify audit trail integrity
    pub fn verify_audit_integrity(&self) -> Result<AuditIntegrityReport> {
        let conn = self.pool.get()
            .map_err(|e| QmsError::Database {
                message: format!("Failed to get database connection: {}", e),
            })?;

        let mut stmt = conn.prepare(
            "SELECT COUNT(*) as total_entries,
                    MIN(timestamp) as earliest_entry,
                    MAX(timestamp) as latest_entry
             FROM audit_trail"
        )?;

        let mut rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;

        if let Some(row) = rows.next() {
            let (total_entries, earliest_entry, latest_entry) = row?;
            
            // Check for gaps in audit trail
            let gaps = self.check_audit_gaps()?;
            
            Ok(AuditIntegrityReport {
                total_entries: total_entries as u64,
                earliest_entry,
                latest_entry,
                integrity_verified: gaps.is_empty(),
                gaps_found: gaps.len(),
                details: if gaps.is_empty() {
                    "Audit trail integrity verified".to_string()
                } else {
                    format!("Found {} potential gaps in audit trail", gaps.len())
                },
            })
        } else {
            Ok(AuditIntegrityReport {
                total_entries: 0,
                earliest_entry: None,
                latest_entry: None,
                integrity_verified: true,
                gaps_found: 0,
                details: "Empty audit trail".to_string(),
            })
        }
    }

    /// Check for gaps in audit trail - Critical for FDA compliance
    fn check_audit_gaps(&self) -> Result<Vec<String>> {
        let conn = self.pool.get()
            .map_err(|e| QmsError::Database {
                message: format!("Failed to get database connection: {}", e),
            })?;

        let mut gaps = Vec::new();

        // Check for temporal gaps (periods longer than expected without entries)
        let mut stmt = conn.prepare(
            "SELECT timestamp, 
                    LAG(timestamp) OVER (ORDER BY timestamp) as prev_timestamp
             FROM audit_trail 
             ORDER BY timestamp"
        )?;
        
        let gap_threshold_hours = 24; // Configurable threshold for suspicious gaps
        
        let rows = stmt.query_map([], |row| {
            let current: String = row.get(0)?;
            let previous: Option<String> = row.get(1)?;
            Ok((current, previous))
        })?;

        for row in rows {
            let (current_str, prev_str) = row?;
            
            if let Some(prev_str) = prev_str {
                if let (Ok(current), Ok(prev)) = (
                    DateTime::parse_from_rfc3339(&current_str),
                    DateTime::parse_from_rfc3339(&prev_str)
                ) {
                    let gap_duration = current.signed_duration_since(prev);
                    
                    if gap_duration.num_hours() > gap_threshold_hours {
                        gaps.push(format!(
                            "Gap of {} hours between {} and {}",
                            gap_duration.num_hours(),
                            prev_str,
                            current_str
                        ));
                    }
                }
            }
        }

        // Check for missing sequence numbers or user sessions without proper start/end
        let mut stmt = conn.prepare(
            "SELECT user_id, session_id, MIN(timestamp) as start_time, MAX(timestamp) as end_time,
                    COUNT(*) as entry_count
             FROM audit_trail 
             GROUP BY user_id, session_id
             HAVING entry_count < 2"
        )?;

        let incomplete_sessions = stmt.query_map([], |row| {
            let user_id: String = row.get(0)?;
            let session_id: String = row.get(1)?;
            let start_time: String = row.get(2)?;
            Ok(format!("Incomplete session for user {} (session {}): started {}", 
                      user_id, session_id, start_time))
        })?;

        for session in incomplete_sessions {
            gaps.push(session?);
        }

        // Check for entries with missing required fields
        let mut stmt = conn.prepare(
            "SELECT id, timestamp FROM audit_trail 
             WHERE user_id IS NULL OR action IS NULL OR resource IS NULL 
                OR outcome IS NULL OR session_id IS NULL"
        )?;

        let invalid_entries = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let timestamp: String = row.get(1)?;
            Ok(format!("Invalid audit entry {} at {}: missing required fields", id, timestamp))
        })?;

        for entry in invalid_entries {
            gaps.push(entry?);
        }

        Ok(gaps)
    }

    /// Create database backup
    pub fn create_backup(&self, backup_path: &str) -> Result<()> {
        let conn = self.pool.get()
            .map_err(|e| QmsError::Database {
                message: format!("Failed to get database connection: {}", e),
            })?;

        let mut backup_conn = Connection::open(backup_path)?;
        let backup = rusqlite::backup::Backup::new(&*conn, &mut backup_conn)?;
        backup.run_to_completion(5, std::time::Duration::from_millis(250), None)?;
        Ok(())
    }
}

/// Audit trail entry from database
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditTrailEntry {
    pub id: String,
    pub timestamp: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub outcome: String,
    pub ip_address: Option<String>,
    pub session_id: String,
    pub metadata: Option<String>,
    pub compliance_version: String,
    pub signature_hash: Option<String>,
    pub created_at: String,
}

/// Audit integrity report
#[derive(Debug, Serialize)]
pub struct AuditIntegrityReport {
    pub total_entries: u64,
    pub earliest_entry: Option<String>,
    pub latest_entry: Option<String>,
    pub integrity_verified: bool,
    pub gaps_found: usize,
    pub details: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::{AuditLogEntry, AuditOutcome};

    #[test]
    fn test_database_initialization() {
        let config = DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false, // Disable WAL for in-memory testing
            backup_interval_hours: 24,
            backup_retention_days: 90,
        };

        let db = Database::new(config);
        assert!(db.is_ok());
    }

    #[test]
    fn test_audit_entry_insertion() {
        let config = DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false,
            backup_interval_hours: 24,
            backup_retention_days: 90,
        };

        let mut db = Database::new(config).unwrap();
        
        let entry = AuditLogEntry::new(
            "user123".to_string(),
            "test_action".to_string(),
            "test_resource".to_string(),
            AuditOutcome::Success,
            "session456".to_string(),
        );

        let result = db.insert_audit_entry(&entry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_trail_retrieval() {
        let config = DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false,
            backup_interval_hours: 24,
            backup_retention_days: 90,
        };

        let mut db = Database::new(config).unwrap();
        
        // Insert test entry
        let entry = AuditLogEntry::new(
            "user123".to_string(),
            "test_action".to_string(),
            "test_resource".to_string(),
            AuditOutcome::Success,
            "session456".to_string(),
        );
        db.insert_audit_entry(&entry).unwrap();

        // Retrieve entries
        let entries = db.get_audit_entries(10, 0, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].user_id, "user123");
    }

    #[test]
    fn test_audit_integrity_verification() {
        let config = DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false,
            backup_interval_hours: 24,
            backup_retention_days: 90,
        };

        let db = Database::new(config).unwrap();
        let report = db.verify_audit_integrity().unwrap();
        
        assert_eq!(report.total_entries, 0);
        assert!(report.integrity_verified);
    }
}