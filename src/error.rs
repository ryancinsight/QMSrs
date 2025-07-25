use thiserror::Error;

/// Custom result type for QMS operations
pub type Result<T> = std::result::Result<T, QmsError>;

/// Comprehensive error types for FDA-compliant QMS system
#[derive(Error, Debug)]
pub enum QmsError {
    /// Database-related errors
    #[error("Database error: {message}")]
    Database { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Audit trail errors (critical for FDA compliance)
    #[error("Audit trail error: {message}")]
    AuditTrail { message: String },

    /// Authentication and authorization errors
    #[error("Security error: {message}")]
    Security { message: String },

    /// Document control errors
    #[error("Document control error: {message}")]
    DocumentControl { message: String },

    /// TUI/Interface errors
    #[error("User interface error: {message}")]
    UserInterface { message: String },

    /// Validation errors for FDA compliance
    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    /// Encryption/Decryption errors
    #[error("Encryption error: {message}")]
    Encryption { message: String },

    /// File system operations errors
    #[error("File system error: {path} - {message}")]
    FileSystem { path: String, message: String },

    /// Network-related errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Serialization/Deserialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    /// Generic application errors
    #[error("Application error: {message}")]
    Application { message: String },
}

impl QmsError {
    /// Get error code for audit logging
    pub fn error_code(&self) -> &'static str {
        match self {
            QmsError::Database { .. } => "DB_ERROR",
            QmsError::Config { .. } => "CFG_ERROR",
            QmsError::AuditTrail { .. } => "AUDIT_ERROR",
            QmsError::Security { .. } => "SEC_ERROR",
            QmsError::DocumentControl { .. } => "DOC_ERROR",
            QmsError::UserInterface { .. } => "UI_ERROR",
            QmsError::Validation { .. } => "VAL_ERROR",
            QmsError::Encryption { .. } => "ENC_ERROR",
            QmsError::FileSystem { .. } => "FS_ERROR",
            QmsError::Network { .. } => "NET_ERROR",
            QmsError::Serialization { .. } => "SER_ERROR",
            QmsError::Application { .. } => "APP_ERROR",
        }
    }

    /// Get severity level for FDA compliance reporting
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            QmsError::AuditTrail { .. } => ErrorSeverity::Critical,
            QmsError::Security { .. } => ErrorSeverity::Critical,
            QmsError::Database { .. } => ErrorSeverity::High,
            QmsError::DocumentControl { .. } => ErrorSeverity::High,
            QmsError::Validation { .. } => ErrorSeverity::Medium,
            QmsError::Encryption { .. } => ErrorSeverity::High,
            QmsError::Config { .. } => ErrorSeverity::Medium,
            QmsError::UserInterface { .. } => ErrorSeverity::Low,
            QmsError::FileSystem { .. } => ErrorSeverity::Medium,
            QmsError::Network { .. } => ErrorSeverity::Medium,
            QmsError::Serialization { .. } => ErrorSeverity::Low,
            QmsError::Application { .. } => ErrorSeverity::Medium,
        }
    }

    /// Check if error requires immediate FDA notification
    pub fn requires_fda_notification(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::Critical)
    }
}

/// Error severity levels for compliance reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "LOW",
            ErrorSeverity::Medium => "MEDIUM",
            ErrorSeverity::High => "HIGH",
            ErrorSeverity::Critical => "CRITICAL",
        }
    }
}

// Convert from common error types
impl From<std::io::Error> for QmsError {
    fn from(err: std::io::Error) -> Self {
        QmsError::FileSystem {
            path: "unknown".to_string(),
            message: err.to_string(),
        }
    }
}

// Database conversion removed for simplification

impl From<serde_json::Error> for QmsError {
    fn from(err: serde_json::Error) -> Self {
        QmsError::Serialization {
            message: err.to_string(),
        }
    }
}

// Config conversion removed for simplification

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(QmsError::Database { message: "test".to_string() }.error_code(), "DB_ERROR");
        assert_eq!(QmsError::AuditTrail { message: "test".to_string() }.error_code(), "AUDIT_ERROR");
        assert_eq!(QmsError::Security { message: "test".to_string() }.error_code(), "SEC_ERROR");
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(
            QmsError::AuditTrail { message: "test".to_string() }.severity(),
            ErrorSeverity::Critical
        );
        assert_eq!(
            QmsError::Security { message: "test".to_string() }.severity(),
            ErrorSeverity::Critical
        );
        assert_eq!(
            QmsError::UserInterface { message: "test".to_string() }.severity(),
            ErrorSeverity::Low
        );
    }

    #[test]
    fn test_fda_notification_requirement() {
        assert!(QmsError::AuditTrail { message: "test".to_string() }.requires_fda_notification());
        assert!(QmsError::Security { message: "test".to_string() }.requires_fda_notification());
        assert!(!QmsError::UserInterface { message: "test".to_string() }.requires_fda_notification());
    }

    #[test]
    fn test_error_severity_as_str() {
        assert_eq!(ErrorSeverity::Low.as_str(), "LOW");
        assert_eq!(ErrorSeverity::Medium.as_str(), "MEDIUM");
        assert_eq!(ErrorSeverity::High.as_str(), "HIGH");
        assert_eq!(ErrorSeverity::Critical.as_str(), "CRITICAL");
    }

    #[test]
    fn test_error_conversion_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let qms_error: QmsError = io_error.into();
        
        match qms_error {
            QmsError::FileSystem { path, message } => {
                assert_eq!(path, "unknown");
                assert!(message.contains("File not found"));
            }
            _ => panic!("Expected FileSystem error"),
        }
    }
}