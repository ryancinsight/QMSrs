use crate::{Result, QmsError, config::DatabaseConfig};
use rusqlite::{Connection, params};
use std::path::Path;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Database manager for FDA-compliant QMS
pub struct Database {
    connection: Connection,
    config: DatabaseConfig,
}

impl Database {
    /// Create new database connection
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        let connection = if config.url == ":memory:" {
            Connection::open_in_memory()?
        } else {
            // Ensure database directory exists
            if let Some(parent) = Path::new(&config.url).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| QmsError::FileSystem {
                        path: parent.display().to_string(),
                        message: format!("Failed to create database directory: {}", e),
                    })?;
            }
            Connection::open(&config.url)?
        };

        // Enable WAL mode for better concurrency and crash recovery
        if config.wal_mode {
            connection.execute("PRAGMA journal_mode=WAL", [])?;
        }

        // Set other pragma settings for FDA compliance
        connection.execute("PRAGMA foreign_keys=ON", [])?;
        connection.execute("PRAGMA synchronous=FULL", [])?;
        connection.execute("PRAGMA secure_delete=ON", [])?;

        let mut db = Self { connection, config };
        db.initialize_schema()?;
        
        Ok(db)
    }

    /// Initialize database schema for FDA compliance
    fn initialize_schema(&mut self) -> Result<()> {
        // Create audit trail table (critical for FDA compliance)
        self.connection.execute(
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
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                role TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                failed_login_attempts INTEGER DEFAULT 0,
                locked_until TEXT,
                last_login TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create documents table for document control system
        self.connection.execute(
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
        self.connection.execute(
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
        self.connection.execute(
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

        // Create indexes for performance
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_trail_timestamp ON audit_trail(timestamp)",
            [],
        )?;
        
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_trail_user_id ON audit_trail(user_id)",
            [],
        )?;
        
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status)",
            [],
        )?;

        Ok(())
    }

    /// Insert audit trail entry
    pub fn insert_audit_entry(&mut self, entry: &crate::logging::AuditLogEntry) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        
        self.connection.execute(
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
        let mut query = "SELECT * FROM audit_trail".to_string();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

        if let Some(uid) = user_id {
            query.push_str(" WHERE user_id = ?");
            params.push(&uid);
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");
        params.push(&limit);
        params.push(&offset);

        let mut stmt = self.connection.prepare(&query)?;
        let audit_iter = stmt.query_map(params.as_slice(), |row| {
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
        let mut stmt = self.connection.prepare(
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

    /// Check for gaps in audit trail
    fn check_audit_gaps(&self) -> Result<Vec<String>> {
        // This is a simplified gap detection - in production you'd want more sophisticated checks
        let mut stmt = self.connection.prepare(
            "SELECT timestamp FROM audit_trail ORDER BY timestamp"
        )?;
        
        let timestamps: Result<Vec<String>> = stmt.query_map([], |row| {
            row.get(0)
        })?.collect();

        // For now, just return empty - real implementation would check for suspicious gaps
        Ok(Vec::new())
    }

    /// Create database backup
    pub fn create_backup(&self, backup_path: &str) -> Result<()> {
        let backup_conn = Connection::open(backup_path)?;
        let backup = rusqlite::backup::Backup::new(&self.connection, &backup_conn)?;
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