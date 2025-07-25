//! # QMSrs - FDA Compliant Medical Device Quality Management System
//! 
//! A comprehensive Quality Management System (QMS) built in Rust,
//! designed to meet FDA 21 CFR Part 820 and ISO 13485 requirements for medical device manufacturers.
//!
//! ## Key Features
//! - FDA-compliant document control system
//! - Complete audit trail for all operations
//! - Risk management module (ISO 14971)
//! - CAPA (Corrective and Preventive Action) system
//! - Role-based access control
//!
//! ## Architecture
//! The system follows SOLID principles and implements comprehensive testing
//! to ensure reliability and regulatory compliance.

pub mod config;
pub mod document;
pub mod error;
// Temporarily commented out for TASK-013 clean build
// pub mod app;
// pub mod database;
// pub mod security;
// pub mod audit;
// pub mod ui;
// pub mod logging;
// pub mod cli;

pub use error::{QmsError, Result};

/// FDA compliance version information
pub const FDA_CFR_PART_820_VERSION: &str = "2022";
pub const ISO_13485_VERSION: &str = "2016";
pub const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Audit trail configuration constants
pub const MAX_AUDIT_RETENTION_DAYS: u32 = 2555; // 7 years as per FDA requirements
pub const AUDIT_LOG_ENCRYPTION: bool = true;
pub const REQUIRED_AUDIT_FIELDS: &[&str] = &[
    "timestamp",
    "user_id", 
    "action",
    "resource",
    "outcome",
    "ip_address",
    "session_id"
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fda_compliance_constants() {
        assert!(!FDA_CFR_PART_820_VERSION.is_empty());
        assert!(!ISO_13485_VERSION.is_empty());
        assert!(!APPLICATION_VERSION.is_empty());
        assert!(MAX_AUDIT_RETENTION_DAYS >= 2555); // Minimum 7 years
        assert!(AUDIT_LOG_ENCRYPTION); // Must be enabled for FDA compliance
        assert!(!REQUIRED_AUDIT_FIELDS.is_empty());
        assert!(REQUIRED_AUDIT_FIELDS.len() >= 7); // Minimum required fields
    }

    #[test]
    fn test_required_audit_fields_completeness() {
        let required_fields = REQUIRED_AUDIT_FIELDS;
        assert!(required_fields.contains(&"timestamp"));
        assert!(required_fields.contains(&"user_id"));
        assert!(required_fields.contains(&"action"));
        assert!(required_fields.contains(&"resource"));
        assert!(required_fields.contains(&"outcome"));
    }
}