use crate::{Result, QmsError};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Document control manager for FDA compliance
pub struct DocumentManager {
    // Database connection would be here in full implementation
}

impl DocumentManager {
    /// Create new document manager
    pub fn new() -> Self {
        Self {}
    }

    /// Create a new controlled document
    pub fn create_document(&mut self, document: Document) -> Result<String> {
        document.validate()?;
        // Implementation would save to database
        Ok(document.id)
    }

    /// Get document by ID
    pub fn get_document(&self, _id: &str) -> Result<Option<Document>> {
        // Implementation would query database
        Ok(None)
    }
}

/// FDA-compliant controlled document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub document_number: String,
    pub title: String,
    pub version: String,
    pub status: DocumentStatus,
    pub document_type: DocumentType,
    pub content_hash: String,
    pub file_path: Option<String>,
    pub created_by: String,
    pub approved_by: Option<String>,
    pub effective_date: Option<DateTime<Utc>>,
    pub review_date: Option<DateTime<Utc>>,
    pub retirement_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    /// Validate document for FDA compliance
    pub fn validate(&self) -> Result<()> {
        if self.document_number.trim().is_empty() {
            return Err(QmsError::DocumentControl {
                message: "Document number is required".to_string(),
            });
        }

        if self.title.trim().is_empty() {
            return Err(QmsError::DocumentControl {
                message: "Document title is required".to_string(),
            });
        }

        Ok(())
    }
}

/// Document status for workflow control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentStatus {
    Draft,
    UnderReview,
    Approved,
    Effective,
    Obsolete,
    Retired,
}

/// Document type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentType {
    SOP,          // Standard Operating Procedure
    WorkInstruction,
    Policy,
    Form,
    Template,
    Specification,
    TestMethod,
    ValidationProtocol,
    Report,
    Manual,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let document = Document {
            id: "doc-001".to_string(),
            document_number: "SOP-001".to_string(),
            title: "Quality Management System Overview".to_string(),
            version: "1.0".to_string(),
            status: DocumentStatus::Draft,
            document_type: DocumentType::SOP,
            content_hash: "abc123".to_string(),
            file_path: None,
            created_by: "user123".to_string(),
            approved_by: None,
            effective_date: None,
            review_date: None,
            retirement_date: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(document.validate().is_ok());
    }

    #[test]
    fn test_document_validation_failure() {
        let mut document = Document {
            id: "doc-001".to_string(),
            document_number: "".to_string(), // Empty document number
            title: "Test Document".to_string(),
            version: "1.0".to_string(),
            status: DocumentStatus::Draft,
            document_type: DocumentType::SOP,
            content_hash: "abc123".to_string(),
            file_path: None,
            created_by: "user123".to_string(),
            approved_by: None,
            effective_date: None,
            review_date: None,
            retirement_date: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(document.validate().is_err());
    }
}