//! CAPA (Corrective and Preventive Action) System
//! 
//! TASK-017: Implement CAPA workflow management
//! 
//! This module implements a comprehensive CAPA system that:
//! - Manages corrective and preventive action workflows
//! - Ensures FDA 21 CFR Part 820 compliance
//! - Integrates with risk management (ISO 14971)
//! - Provides complete audit trail capabilities
//! - Supports investigation tracking and effectiveness verification
//!
//! The implementation follows SOLID principles:
//! - Single Responsibility: Each component has one clear purpose
//! - Open/Closed: Extensible for new CAPA types and workflows
//! - Liskov Substitution: Proper trait implementations
//! - Interface Segregation: Focused interfaces for different concerns
//! - Dependency Inversion: Abstract interfaces over concrete implementations

use crate::error::{QmsError, Result};
use crate::audit::AuditManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// CAPA Status following FDA workflow requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CapaStatus {
    /// Initial identification and documentation
    Identified,
    /// Investigation in progress
    InvestigationInProgress,
    /// Root cause analysis phase
    RootCauseAnalysis,
    /// Corrective actions being implemented
    CorrectiveActionInProgress,
    /// Preventive actions being implemented
    PreventiveActionInProgress,
    /// Effectiveness verification phase
    EffectivenessVerification,
    /// CAPA completed successfully
    Closed,
    /// CAPA cancelled or rejected
    Cancelled,
}

impl CapaStatus {
    /// Get human-readable status description
    pub fn as_str(&self) -> &'static str {
        match self {
            CapaStatus::Identified => "Identified",
            CapaStatus::InvestigationInProgress => "Investigation In Progress",
            CapaStatus::RootCauseAnalysis => "Root Cause Analysis",
            CapaStatus::CorrectiveActionInProgress => "Corrective Action In Progress",
            CapaStatus::PreventiveActionInProgress => "Preventive Action In Progress",
            CapaStatus::EffectivenessVerification => "Effectiveness Verification",
            CapaStatus::Closed => "Closed",
            CapaStatus::Cancelled => "Cancelled",
        }
    }

    /// Check if status allows state transitions
    pub fn can_transition_to(&self, new_status: &CapaStatus) -> bool {
        match (self, new_status) {
            (CapaStatus::Identified, CapaStatus::InvestigationInProgress) => true,
            (CapaStatus::InvestigationInProgress, CapaStatus::RootCauseAnalysis) => true,
            (CapaStatus::RootCauseAnalysis, CapaStatus::CorrectiveActionInProgress) => true,
            (CapaStatus::RootCauseAnalysis, CapaStatus::PreventiveActionInProgress) => true,
            (CapaStatus::CorrectiveActionInProgress, CapaStatus::EffectivenessVerification) => true,
            (CapaStatus::PreventiveActionInProgress, CapaStatus::EffectivenessVerification) => true,
            (CapaStatus::EffectivenessVerification, CapaStatus::Closed) => true,
            (_, CapaStatus::Cancelled) => true, // Can cancel from any state
            _ => false,
        }
    }
}

/// CAPA Priority levels for resource allocation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CapaPriority {
    Critical,   // Immediate safety concerns
    High,       // Significant quality impact
    Medium,     // Moderate impact
    Low,        // Minor issues
}

impl CapaPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            CapaPriority::Critical => "Critical",
            CapaPriority::High => "High",
            CapaPriority::Medium => "Medium",
            CapaPriority::Low => "Low",
        }
    }
}

/// CAPA Type classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CapaType {
    Corrective,    // Address existing problems
    Preventive,    // Prevent potential problems
    Combined,      // Both corrective and preventive
}

/// Core CAPA record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapaRecord {
    pub id: String,
    pub title: String,
    pub description: String,
    pub capa_type: CapaType,
    pub priority: CapaPriority,
    pub status: CapaStatus,
    pub initiator_id: String,
    pub assigned_to: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub closed_date: Option<DateTime<Utc>>,
    pub source_document: Option<String>,
    pub related_risk_id: Option<String>,
    pub investigation_summary: Option<String>,
    pub root_cause: Option<String>,
    pub corrective_actions: Vec<CapaAction>,
    pub preventive_actions: Vec<CapaAction>,
    pub effectiveness_verification: Option<EffectivenessVerification>,
    pub metadata: HashMap<String, String>,
}

/// Individual action within a CAPA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapaAction {
    pub id: String,
    pub description: String,
    pub assigned_to: String,
    pub due_date: DateTime<Utc>,
    pub completed_date: Option<DateTime<Utc>>,
    pub verification_method: String,
    pub status: ActionStatus,
    pub evidence: Vec<String>, // File paths or references
}

/// Action completion status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionStatus {
    Planned,
    InProgress,
    Completed,
    Verified,
    Overdue,
}

/// Effectiveness verification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessVerification {
    pub verification_date: DateTime<Utc>,
    pub verifier_id: String,
    pub method: String,
    pub results: String,
    pub is_effective: bool,
    pub follow_up_required: bool,
    pub follow_up_actions: Vec<String>,
}

/// CAPA workflow management service
pub struct CapaService {
    audit_manager: AuditManager,
}

impl CapaService {
    /// Create new CAPA service with audit integration
    pub fn new(audit_manager: AuditManager) -> Self {
        Self { audit_manager }
    }

    /// Create a new CAPA record
    pub fn create_capa(&self, 
        title: String,
        description: String,
        capa_type: CapaType,
        priority: CapaPriority,
        initiator_id: String,
        assigned_to: String,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<CapaRecord> {
        let capa_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let capa = CapaRecord {
            id: capa_id.clone(),
            title: title.clone(),
            description,
            capa_type: capa_type.clone(),
            priority: priority.clone(),
            status: CapaStatus::Identified,
            initiator_id: initiator_id.clone(),
            assigned_to: assigned_to.clone(),
            created_at: now,
            updated_at: now,
            due_date,
            closed_date: None,
            source_document: None,
            related_risk_id: None,
            investigation_summary: None,
            root_cause: None,
            corrective_actions: Vec::new(),
            preventive_actions: Vec::new(),
            effectiveness_verification: None,
            metadata: HashMap::new(),
        };

        // Audit trail for CAPA creation
        self.audit_manager.log_action(
            &initiator_id,
            "capa_created",
            &format!("capa:{}", capa_id),
            "Success",
            Some(format!("Created {} CAPA: {} (Priority: {})", 
                capa_type.as_str(), title, priority.as_str())),
        )?;

        Ok(capa)
    }

    /// Update CAPA status with validation
    pub fn update_status(&self, 
        capa: &mut CapaRecord, 
        new_status: CapaStatus, 
        user_id: &str,
        comment: Option<String>,
    ) -> Result<()> {
        // Validate status transition
        if !capa.status.can_transition_to(&new_status) {
            return Err(QmsError::ValidationError {
                field: "status".to_string(),
                message: format!("Invalid status transition from {} to {}", 
                    capa.status.as_str(), new_status.as_str()),
            });
        }

        let old_status = capa.status.clone();
        capa.status = new_status.clone();
        capa.updated_at = Utc::now();

        // Set closed date if completing
        if new_status == CapaStatus::Closed {
            capa.closed_date = Some(Utc::now());
        }

        // Audit trail for status change
        let audit_message = match comment {
            Some(c) => format!("Status changed from {} to {}: {}", 
                old_status.as_str(), new_status.as_str(), c),
            None => format!("Status changed from {} to {}", 
                old_status.as_str(), new_status.as_str()),
        };

        self.audit_manager.log_action(
            user_id,
            "capa_status_updated",
            &format!("capa:{}", capa.id),
            "Success",
            Some(audit_message),
        )?;

        Ok(())
    }

    /// Add corrective action to CAPA
    pub fn add_corrective_action(&self,
        capa: &mut CapaRecord,
        description: String,
        assigned_to: String,
        due_date: DateTime<Utc>,
        verification_method: String,
        user_id: &str,
    ) -> Result<String> {
        let action_id = Uuid::new_v4().to_string();
        
        let action = CapaAction {
            id: action_id.clone(),
            description: description.clone(),
            assigned_to: assigned_to.clone(),
            due_date,
            completed_date: None,
            verification_method,
            status: ActionStatus::Planned,
            evidence: Vec::new(),
        };

        capa.corrective_actions.push(action);
        capa.updated_at = Utc::now();

        // Audit trail
        self.audit_manager.log_action(
            user_id,
            "corrective_action_added",
            &format!("capa:{}", capa.id),
            "Success",
            Some(format!("Added corrective action: {} (Assigned to: {})", 
                description, assigned_to)),
        )?;

        Ok(action_id)
    }

    /// Add preventive action to CAPA
    pub fn add_preventive_action(&self,
        capa: &mut CapaRecord,
        description: String,
        assigned_to: String,
        due_date: DateTime<Utc>,
        verification_method: String,
        user_id: &str,
    ) -> Result<String> {
        let action_id = Uuid::new_v4().to_string();
        
        let action = CapaAction {
            id: action_id.clone(),
            description: description.clone(),
            assigned_to: assigned_to.clone(),
            due_date,
            completed_date: None,
            verification_method,
            status: ActionStatus::Planned,
            evidence: Vec::new(),
        };

        capa.preventive_actions.push(action);
        capa.updated_at = Utc::now();

        // Audit trail
        self.audit_manager.log_action(
            user_id,
            "preventive_action_added",
            &format!("capa:{}", capa.id),
            "Success",
            Some(format!("Added preventive action: {} (Assigned to: {})", 
                description, assigned_to)),
        )?;

        Ok(action_id)
    }

    /// Complete an action within a CAPA
    pub fn complete_action(&self,
        capa: &mut CapaRecord,
        action_id: &str,
        completion_evidence: Vec<String>,
        user_id: &str,
    ) -> Result<()> {
        let now = Utc::now();
        let mut action_found = false;

        // Update corrective actions
        for action in &mut capa.corrective_actions {
            if action.id == action_id {
                action.status = ActionStatus::Completed;
                action.completed_date = Some(now);
                action.evidence = completion_evidence.clone();
                action_found = true;
                break;
            }
        }

        // Update preventive actions if not found in corrective
        if !action_found {
            for action in &mut capa.preventive_actions {
                if action.id == action_id {
                    action.status = ActionStatus::Completed;
                    action.completed_date = Some(now);
                    action.evidence = completion_evidence.clone();
                    action_found = true;
                    break;
                }
            }
        }

        if !action_found {
            return Err(QmsError::NotFound {
                resource: "action".to_string(),
                id: action_id.to_string(),
            });
        }

        capa.updated_at = now;

        // Audit trail
        self.audit_manager.log_action(
            user_id,
            "action_completed",
            &format!("capa:{}/action:{}", capa.id, action_id),
            "Success",
            Some(format!("Action completed with {} evidence items", 
                completion_evidence.len())),
        )?;

        Ok(())
    }

    /// Verify effectiveness of CAPA
    pub fn verify_effectiveness(&self,
        capa: &mut CapaRecord,
        verification_method: String,
        results: String,
        is_effective: bool,
        verifier_id: String,
        follow_up_actions: Vec<String>,
    ) -> Result<()> {
        let verification = EffectivenessVerification {
            verification_date: Utc::now(),
            verifier_id: verifier_id.clone(),
            method: verification_method,
            results: results.clone(),
            is_effective,
            follow_up_required: !follow_up_actions.is_empty(),
            follow_up_actions,
        };

        capa.effectiveness_verification = Some(verification);
        capa.updated_at = Utc::now();

        // Audit trail
        self.audit_manager.log_action(
            &verifier_id,
            "effectiveness_verified",
            &format!("capa:{}", capa.id),
            "Success",
            Some(format!("Effectiveness verification: {} (Effective: {})", 
                results, is_effective)),
        )?;

        Ok(())
    }

    /// Get CAPA metrics for reporting
    pub fn get_capa_metrics(&self, capas: &[CapaRecord]) -> CapaMetrics {
        let total_count = capas.len();
        let mut status_counts = HashMap::new();
        let mut priority_counts = HashMap::new();
        let mut overdue_count = 0;
        let now = Utc::now();

        for capa in capas {
            // Count by status
            let status_str = capa.status.as_str();
            *status_counts.entry(status_str.to_string()).or_insert(0) += 1;

            // Count by priority
            let priority_str = capa.priority.as_str();
            *priority_counts.entry(priority_str.to_string()).or_insert(0) += 1;

            // Check if overdue
            if let Some(due_date) = capa.due_date {
                if due_date < now && capa.status != CapaStatus::Closed {
                    overdue_count += 1;
                }
            }
        }

        let closed_count = status_counts.get("Closed").copied().unwrap_or(0);

        CapaMetrics {
            total_count,
            status_counts,
            priority_counts,
            overdue_count,
            closed_count,
        }
    }
}

/// CAPA metrics for reporting and dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct CapaMetrics {
    pub total_count: usize,
    pub status_counts: HashMap<String, usize>,
    pub priority_counts: HashMap<String, usize>,
    pub overdue_count: usize,
    pub closed_count: usize,
}

// Trait implementations for enum conversions
impl CapaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CapaType::Corrective => "Corrective",
            CapaType::Preventive => "Preventive",
            CapaType::Combined => "Combined",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    fn setup_test_service() -> CapaService {
        let config = crate::config::DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false,
            backup_interval_hours: 24,
            backup_retention_days: 90,
        };
        let database = crate::database::Database::new(config).unwrap();
        let audit_manager = AuditManager::new(database);
        CapaService::new(audit_manager)
    }

    #[test]
    fn test_create_capa_corrective() {
        let service = setup_test_service();
        
        let capa = service.create_capa(
            "Critical Safety Issue".to_string(),
            "Device malfunction causing safety concern".to_string(),
            CapaType::Corrective,
            CapaPriority::Critical,
            "user123".to_string(),
            "engineer456".to_string(),
            Some(Utc::now() + chrono::Duration::days(30)),
        ).unwrap();

        assert_eq!(capa.title, "Critical Safety Issue");
        assert_eq!(capa.capa_type, CapaType::Corrective);
        assert_eq!(capa.priority, CapaPriority::Critical);
        assert_eq!(capa.status, CapaStatus::Identified);
        assert_eq!(capa.initiator_id, "user123");
        assert_eq!(capa.assigned_to, "engineer456");
        assert!(capa.due_date.is_some());
    }

    #[test]
    fn test_status_transition_validation() {
        let status = CapaStatus::Identified;
        
        // Valid transition
        assert!(status.can_transition_to(&CapaStatus::InvestigationInProgress));
        
        // Invalid transition
        assert!(!status.can_transition_to(&CapaStatus::Closed));
        
        // Can always cancel
        assert!(status.can_transition_to(&CapaStatus::Cancelled));
    }

    #[test]
    fn test_update_status_valid() {
        let service = setup_test_service();
        let mut capa = service.create_capa(
            "Test CAPA".to_string(),
            "Test description".to_string(),
            CapaType::Corrective,
            CapaPriority::Medium,
            "user123".to_string(),
            "engineer456".to_string(),
            None,
        ).unwrap();

        let result = service.update_status(
            &mut capa,
            CapaStatus::InvestigationInProgress,
            "user123",
            Some("Starting investigation".to_string()),
        );

        assert!(result.is_ok());
        assert_eq!(capa.status, CapaStatus::InvestigationInProgress);
    }

    #[test]
    fn test_update_status_invalid_transition() {
        let service = setup_test_service();
        let mut capa = service.create_capa(
            "Test CAPA".to_string(),
            "Test description".to_string(),
            CapaType::Corrective,
            CapaPriority::Medium,
            "user123".to_string(),
            "engineer456".to_string(),
            None,
        ).unwrap();

        let result = service.update_status(
            &mut capa,
            CapaStatus::Closed,
            "user123",
            None,
        );

        assert!(result.is_err());
        assert_eq!(capa.status, CapaStatus::Identified); // Should remain unchanged
    }

    #[test]
    fn test_add_corrective_action() {
        let service = setup_test_service();
        let mut capa = service.create_capa(
            "Test CAPA".to_string(),
            "Test description".to_string(),
            CapaType::Corrective,
            CapaPriority::Medium,
            "user123".to_string(),
            "engineer456".to_string(),
            None,
        ).unwrap();

        let action_id = service.add_corrective_action(
            &mut capa,
            "Fix the issue".to_string(),
            "engineer456".to_string(),
            Utc::now() + chrono::Duration::days(14),
            "Testing and validation".to_string(),
            "user123",
        ).unwrap();

        assert_eq!(capa.corrective_actions.len(), 1);
        assert_eq!(capa.corrective_actions[0].id, action_id);
        assert_eq!(capa.corrective_actions[0].description, "Fix the issue");
        assert_eq!(capa.corrective_actions[0].status, ActionStatus::Planned);
    }

    #[test]
    fn test_complete_action() {
        let service = setup_test_service();
        let mut capa = service.create_capa(
            "Test CAPA".to_string(),
            "Test description".to_string(),
            CapaType::Corrective,
            CapaPriority::Medium,
            "user123".to_string(),
            "engineer456".to_string(),
            None,
        ).unwrap();

        let action_id = service.add_corrective_action(
            &mut capa,
            "Fix the issue".to_string(),
            "engineer456".to_string(),
            Utc::now() + chrono::Duration::days(14),
            "Testing and validation".to_string(),
            "user123",
        ).unwrap();

        let evidence = vec!["test_report.pdf".to_string(), "validation_results.xlsx".to_string()];
        let result = service.complete_action(
            &mut capa,
            &action_id,
            evidence.clone(),
            "engineer456",
        );

        assert!(result.is_ok());
        assert_eq!(capa.corrective_actions[0].status, ActionStatus::Completed);
        assert!(capa.corrective_actions[0].completed_date.is_some());
        assert_eq!(capa.corrective_actions[0].evidence, evidence);
    }

    #[test]
    fn test_verify_effectiveness() {
        let service = setup_test_service();
        let mut capa = service.create_capa(
            "Test CAPA".to_string(),
            "Test description".to_string(),
            CapaType::Corrective,
            CapaPriority::Medium,
            "user123".to_string(),
            "engineer456".to_string(),
            None,
        ).unwrap();

        let result = service.verify_effectiveness(
            &mut capa,
            "Statistical analysis".to_string(),
            "Defect rate reduced by 95%".to_string(),
            true,
            "qa_manager".to_string(),
            vec![],
        );

        assert!(result.is_ok());
        assert!(capa.effectiveness_verification.is_some());
        
        let verification = capa.effectiveness_verification.unwrap();
        assert_eq!(verification.verifier_id, "qa_manager");
        assert_eq!(verification.is_effective, true);
        assert_eq!(verification.follow_up_required, false);
    }

    #[test]
    fn test_capa_metrics() {
        let service = setup_test_service();
        
        let mut capas = vec![
            service.create_capa(
                "CAPA 1".to_string(),
                "Description 1".to_string(),
                CapaType::Corrective,
                CapaPriority::Critical,
                "user1".to_string(),
                "eng1".to_string(),
                Some(Utc::now() - chrono::Duration::days(1)), // Overdue
            ).unwrap(),
            service.create_capa(
                "CAPA 2".to_string(),
                "Description 2".to_string(),
                CapaType::Preventive,
                CapaPriority::High,
                "user2".to_string(),
                "eng2".to_string(),
                Some(Utc::now() + chrono::Duration::days(10)),
            ).unwrap(),
        ];

        // Follow proper workflow to close one CAPA
        service.update_status(&mut capas[1], CapaStatus::InvestigationInProgress, "user2", None).unwrap();
        service.update_status(&mut capas[1], CapaStatus::RootCauseAnalysis, "user2", None).unwrap();
        service.update_status(&mut capas[1], CapaStatus::CorrectiveActionInProgress, "user2", None).unwrap();
        service.update_status(&mut capas[1], CapaStatus::EffectivenessVerification, "user2", None).unwrap();
        service.update_status(&mut capas[1], CapaStatus::Closed, "user2", None).unwrap();

        let metrics = service.get_capa_metrics(&capas);

        assert_eq!(metrics.total_count, 2);
        assert_eq!(metrics.overdue_count, 1);
        assert_eq!(metrics.closed_count, 1);
        assert_eq!(metrics.priority_counts.get("Critical"), Some(&1));
        assert_eq!(metrics.priority_counts.get("High"), Some(&1));
    }

    #[test]
    fn test_capa_priority_levels() {
        assert_eq!(CapaPriority::Critical.as_str(), "Critical");
        assert_eq!(CapaPriority::High.as_str(), "High");
        assert_eq!(CapaPriority::Medium.as_str(), "Medium");
        assert_eq!(CapaPriority::Low.as_str(), "Low");
    }

    #[test]
    fn test_capa_type_classification() {
        assert_eq!(CapaType::Corrective.as_str(), "Corrective");
        assert_eq!(CapaType::Preventive.as_str(), "Preventive");
        assert_eq!(CapaType::Combined.as_str(), "Combined");
    }

    #[test]
    fn test_action_status_workflow() {
        let action_status = ActionStatus::Planned;
        assert_eq!(action_status, ActionStatus::Planned);
        
        // Test all status variants exist
        let _in_progress = ActionStatus::InProgress;
        let _completed = ActionStatus::Completed;
        let _verified = ActionStatus::Verified;
        let _overdue = ActionStatus::Overdue;
    }
}