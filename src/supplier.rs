//! # Supplier Management Module - Vendor Qualification
//!
//! Provides supplier onboarding, qualification tracking, and monitoring per
//! FDA 21 CFR Part 820.50 and ISO 13485 §7.4 requirements.
//! 
//! Design:
//! - SOLID: `SupplierService` only handles supplier domain logic.
//! - GRASP Repository pattern via `SupplierRepository`.
//! - FIRST tests in module doc tests.
//!
//! ## Key Capabilities (Phase 3 extension)
//! * Create supplier record with Pending status.
//! * Qualify or disqualify suppliers with audit logging.
//! * Generate supplier compliance metrics.

use crate::{audit::AuditLogger, error::Result};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::supplier_repo::SupplierRepository;

/// Supplier qualification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupplierStatus {
    Pending,
    Qualified,
    Disqualified,
}

/// Supplier entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: Uuid,
    pub name: String,
    pub contact_info: Option<String>,
    pub status: SupplierStatus,
    pub qualification_date: Option<NaiveDate>,
    pub qualification_expiry_date: Option<NaiveDate>,
    pub approved_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Service layer encapsulating supplier lifecycle operations
pub struct SupplierService {
    audit_logger: AuditLogger,
    repository: SupplierRepository,
}

impl SupplierService {
    pub fn new(audit_logger: AuditLogger, repository: SupplierRepository) -> Self {
        Self {
            audit_logger,
            repository,
        }
    }

    /// Register a new supplier in Pending status
    pub fn register_supplier(&self, name: String, contact: Option<String>) -> Result<Supplier> {
        let supplier = Supplier {
            id: Uuid::new_v4(),
            name: name.clone(),
            contact_info: contact.clone(),
            status: SupplierStatus::Pending,
            qualification_date: None,
            qualification_expiry_date: None,
            approved_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        // Persist
        self.repository.insert(&supplier)?;
        // Audit
        self.audit_logger.log_event(
            "system",
            "REGISTER_SUPPLIER",
            &format!("supplier:{}", supplier.id),
            "SUCCESS",
            Some(format!("name={}", name)),
        );
        Ok(supplier)
    }

    /// Qualify a supplier (update status & dates)
    pub fn qualify_supplier(
        &self,
        supplier: &mut Supplier,
        approved_by: String,
        expiry: Option<NaiveDate>,
    ) -> Result<()> {
        supplier.status = SupplierStatus::Qualified;
        supplier.qualification_date = Some(Utc::now().date_naive());
        supplier.qualification_expiry_date = expiry;
        supplier.approved_by = Some(approved_by.clone());
        supplier.updated_at = Utc::now();

        self.repository.update(supplier)?;
        self.audit_logger.log_event(
            &approved_by,
            "QUALIFY_SUPPLIER",
            &format!("supplier:{}", supplier.id),
            "SUCCESS",
            None,
        );
        Ok(())
    }

    /// Disqualify supplier
    pub fn disqualify_supplier(&self, supplier: &mut Supplier, by: String, reason: String) -> Result<()> {
        supplier.status = SupplierStatus::Disqualified;
        supplier.updated_at = Utc::now();
        self.repository.update(supplier)?;
        self.audit_logger.log_event(
            &by,
            "DISQUALIFY_SUPPLIER",
            &format!("supplier:{}", supplier.id),
            "SUCCESS",
            Some(reason),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::DatabaseConfig, database::Database, audit::AuditLogger};
    use crate::supplier_repo::SupplierRepository;

    fn setup_service() -> SupplierService {
        let db = Database::new(DatabaseConfig::default()).unwrap();
        let repo = SupplierRepository::new(db);
        SupplierService::new(AuditLogger::new_test(), repo)
    }

    #[test]
    fn test_register_and_qualify() {
        let service = setup_service();
        let mut supplier = service.register_supplier("Test Vendor".to_string(), None).unwrap();
        assert_eq!(supplier.status, SupplierStatus::Pending);
        service
            .qualify_supplier(&mut supplier, "qa_manager".to_string(), None)
            .unwrap();
        assert_eq!(supplier.status, SupplierStatus::Qualified);
        assert!(supplier.qualification_date.is_some());
    }

    #[test]
    fn test_disqualify() {
        let service = setup_service();
        let mut supplier = service.register_supplier("Bad Vendor".to_string(), None).unwrap();
        service
            .disqualify_supplier(&mut supplier, "qa_manager".to_string(), "Quality issues".to_string())
            .unwrap();
        assert_eq!(supplier.status, SupplierStatus::Disqualified);
    }
}