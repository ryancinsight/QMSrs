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

/// Supplier compliance metrics structure
/// Provides aggregated counts for dashboard & API usage.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SupplierMetrics {
    /// Total number of suppliers in system
    pub total_count: usize,
    /// Suppliers with `Qualified` status
    pub qualified_count: usize,
    /// Suppliers currently in `Pending` status
    pub pending_count: usize,
    /// Suppliers that were `Disqualified`
    pub disqualified_count: usize,
    /// Percentage of qualified suppliers (0.0-100.0)
    pub qualified_percentage: f64,
}

impl SupplierMetrics {
    /// Compute metrics from slice of suppliers – FAST/ISOLATED helper.
    pub fn from_suppliers(suppliers: &[Supplier]) -> Self {
        let total_count = suppliers.len();
        let qualified_count = suppliers
            .iter()
            .filter(|s| s.status == SupplierStatus::Qualified)
            .count();
        let pending_count = suppliers
            .iter()
            .filter(|s| s.status == SupplierStatus::Pending)
            .count();
        let disqualified_count = suppliers
            .iter()
            .filter(|s| s.status == SupplierStatus::Disqualified)
            .count();
        let qualified_percentage = if total_count == 0 {
            0.0
        } else {
            (qualified_count as f64 / total_count as f64) * 100.0
        };

        Self {
            total_count,
            qualified_count,
            pending_count,
            disqualified_count,
            qualified_percentage,
        }
    }
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

    #[test]
    fn test_supplier_metrics_calculation() {
        let mut suppliers = Vec::new();
        // Pending supplier
        suppliers.push(Supplier {
            id: Uuid::new_v4(),
            name: "Pending".to_string(),
            contact_info: None,
            status: SupplierStatus::Pending,
            qualification_date: None,
            qualification_expiry_date: None,
            approved_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        // Qualified supplier
        suppliers.push(Supplier {
            id: Uuid::new_v4(),
            name: "Qualified".to_string(),
            contact_info: None,
            status: SupplierStatus::Qualified,
            qualification_date: None,
            qualification_expiry_date: None,
            approved_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
        // Disqualified supplier
        suppliers.push(Supplier {
            id: Uuid::new_v4(),
            name: "Disqualified".to_string(),
            contact_info: None,
            status: SupplierStatus::Disqualified,
            qualification_date: None,
            qualification_expiry_date: None,
            approved_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        let metrics = SupplierMetrics::from_suppliers(&suppliers);
        assert_eq!(metrics.total_count, 3);
        assert_eq!(metrics.qualified_count, 1);
        assert_eq!(metrics.pending_count, 1);
        assert_eq!(metrics.disqualified_count, 1);
        assert_eq!(metrics.qualified_percentage, (1.0 / 3.0) * 100.0);
    }
}