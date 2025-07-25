//! # Risk Management Module - ISO 14971 Compliance
//! 
//! This module implements comprehensive risk management capabilities 
//! compliant with ISO 14971 (Medical Device Risk Management).
//!
//! ## Key Features
//! - Risk assessment creation and management
//! - Risk matrix calculations (severity × probability)
//! - Risk control measures tracking
//! - Residual risk evaluation
//! - Risk management file maintenance
//! - Complete audit trail integration

use crate::error::{QmsError, Result};
use crate::audit::AuditLogger;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// ISO 14971 Risk Severity levels (1-5 scale)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RiskSeverity {
    Negligible = 1,
    Minor = 2,
    Serious = 3,
    Critical = 4,
    Catastrophic = 5,
}

/// ISO 14971 Risk Probability levels (1-5 scale)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RiskProbability {
    Remote = 1,
    Unlikely = 2,
    Possible = 3,
    Probable = 4,
    Frequent = 5,
}

/// ISO 14971 Risk Acceptability levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskAcceptability {
    Acceptable,
    Tolerable,
    Unacceptable,
}

/// ISO 14971 Risk Control Measure types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMeasureType {
    InherentSafety,
    ProtectiveMeasures,
    Information,
}

/// Risk Assessment according to ISO 14971
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub id: Uuid,
    pub device_name: String,
    pub hazard_description: String,
    pub hazardous_situation: String,
    pub foreseeable_sequence: String,
    pub harm_description: String,
    pub initial_severity: RiskSeverity,
    pub initial_probability: RiskProbability,
    pub initial_risk_level: u8,
    pub acceptability: RiskAcceptability,
    pub control_measures: Vec<ControlMeasure>,
    pub residual_severity: Option<RiskSeverity>,
    pub residual_probability: Option<RiskProbability>,
    pub residual_risk_level: Option<u8>,
    pub residual_acceptability: Option<RiskAcceptability>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub status: RiskAssessmentStatus,
}

/// Risk Control Measure according to ISO 14971
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMeasure {
    pub id: Uuid,
    pub risk_assessment_id: Uuid,
    pub measure_type: ControlMeasureType,
    pub description: String,
    pub implementation_details: String,
    pub effectiveness_verification: String,
    pub verification_status: VerificationStatus,
    pub implemented_by: String,
    pub implemented_at: DateTime<Utc>,
    pub verified_by: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
}

/// Risk Assessment Status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskAssessmentStatus {
    Draft,
    UnderReview,
    Approved,
    RequiresUpdate,
    Archived,
}

/// Control Measure Verification Status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    InProgress,
    Verified,
    Failed,
    RequiresReview,
}

/// Risk Management Service implementing ISO 14971
pub struct RiskManagementService {
    audit_logger: AuditLogger,
}

impl RiskManagementService {
    /// Create new Risk Management Service
    pub fn new(audit_logger: AuditLogger) -> Self {
        Self { audit_logger }
    }

    /// Create new risk assessment (ISO 14971 compliant)
    pub async fn create_risk_assessment(
        &self,
        device_name: String,
        hazard_description: String,
        hazardous_situation: String,
        foreseeable_sequence: String,
        harm_description: String,
        initial_severity: RiskSeverity,
        initial_probability: RiskProbability,
        created_by: String,
    ) -> Result<RiskAssessment> {
        let id = Uuid::new_v4();
        let initial_risk_level = self.calculate_risk_level(initial_severity, initial_probability);
        let acceptability = self.determine_acceptability(initial_risk_level);

        let assessment = RiskAssessment {
            id,
            device_name: device_name.clone(),
            hazard_description: hazard_description.clone(),
            hazardous_situation,
            foreseeable_sequence,
            harm_description,
            initial_severity,
            initial_probability,
            initial_risk_level,
            acceptability,
            control_measures: Vec::new(),
            residual_severity: None,
            residual_probability: None,
            residual_risk_level: None,
            residual_acceptability: None,
            created_by: created_by.clone(),
            created_at: Utc::now(),
            updated_by: None,
            updated_at: None,
            reviewed_by: None,
            reviewed_at: None,
            status: RiskAssessmentStatus::Draft,
        };

        // Log audit event
        self.audit_logger.log_event(
            &created_by,
            "CREATE_RISK_ASSESSMENT",
            &format!("risk_assessment:{}", id),
            "SUCCESS",
            Some(format!("Created risk assessment for device: {}", device_name)),
        ).await?;

        Ok(assessment)
    }

    /// Add control measure to risk assessment
    pub async fn add_control_measure(
        &self,
        risk_assessment_id: Uuid,
        measure_type: ControlMeasureType,
        description: String,
        implementation_details: String,
        effectiveness_verification: String,
        implemented_by: String,
    ) -> Result<ControlMeasure> {
        let id = Uuid::new_v4();

        let control_measure = ControlMeasure {
            id,
            risk_assessment_id,
            measure_type,
            description: description.clone(),
            implementation_details,
            effectiveness_verification,
            verification_status: VerificationStatus::Pending,
            implemented_by: implemented_by.clone(),
            implemented_at: Utc::now(),
            verified_by: None,
            verified_at: None,
        };

        // Log audit event
        self.audit_logger.log_event(
            &implemented_by,
            "ADD_CONTROL_MEASURE",
            &format!("control_measure:{}", id),
            "SUCCESS",
            Some(format!("Added control measure: {}", description)),
        ).await?;

        Ok(control_measure)
    }

    /// Calculate residual risk after control measures
    pub async fn calculate_residual_risk(
        &self,
        risk_assessment: &mut RiskAssessment,
        residual_severity: RiskSeverity,
        residual_probability: RiskProbability,
        calculated_by: String,
    ) -> Result<()> {
        let residual_risk_level = self.calculate_risk_level(residual_severity, residual_probability);
        let residual_acceptability = self.determine_acceptability(residual_risk_level);

        risk_assessment.residual_severity = Some(residual_severity);
        risk_assessment.residual_probability = Some(residual_probability);
        risk_assessment.residual_risk_level = Some(residual_risk_level);
        risk_assessment.residual_acceptability = Some(residual_acceptability);
        risk_assessment.updated_by = Some(calculated_by.clone());
        risk_assessment.updated_at = Some(Utc::now());

        // Log audit event
        self.audit_logger.log_event(
            &calculated_by,
            "CALCULATE_RESIDUAL_RISK",
            &format!("risk_assessment:{}", risk_assessment.id),
            "SUCCESS",
            Some(format!("Calculated residual risk level: {}", residual_risk_level)),
        ).await?;

        Ok(())
    }

    /// Approve risk assessment (requires review)
    pub async fn approve_risk_assessment(
        &self,
        risk_assessment: &mut RiskAssessment,
        reviewed_by: String,
    ) -> Result<()> {
        // Validation: Control measures must exist and be verified for unacceptable risks
        if risk_assessment.acceptability == RiskAcceptability::Unacceptable {
            if risk_assessment.control_measures.is_empty() {
                return Err(QmsError::Validation {
                    field: "control_measures".to_string(),
                    message: "Unacceptable risks must have control measures before approval".to_string(),
                });
            }
            for measure in &risk_assessment.control_measures {
                if measure.verification_status != VerificationStatus::Verified {
                    return Err(QmsError::Validation {
                        field: "verification_status".to_string(),
                        message: "All control measures must be verified before approval".to_string(),
                    });
                }
            }
        }

        risk_assessment.status = RiskAssessmentStatus::Approved;
        risk_assessment.reviewed_by = Some(reviewed_by.clone());
        risk_assessment.reviewed_at = Some(Utc::now());

        // Log audit event
        self.audit_logger.log_event(
            &reviewed_by,
            "APPROVE_RISK_ASSESSMENT",
            &format!("risk_assessment:{}", risk_assessment.id),
            "SUCCESS",
            Some("Risk assessment approved".to_string()),
        ).await?;

        Ok(())
    }

    /// Verify control measure effectiveness
    pub async fn verify_control_measure(
        &self,
        control_measure: &mut ControlMeasure,
        verified_by: String,
        verification_successful: bool,
    ) -> Result<()> {
        control_measure.verification_status = if verification_successful {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Failed
        };
        control_measure.verified_by = Some(verified_by.clone());
        control_measure.verified_at = Some(Utc::now());

        let outcome = if verification_successful { "SUCCESS" } else { "FAILED" };
        
        // Log audit event
        self.audit_logger.log_event(
            &verified_by,
            "VERIFY_CONTROL_MEASURE",
            &format!("control_measure:{}", control_measure.id),
            outcome,
            Some(format!("Control measure verification: {}", outcome)),
        ).await?;

        Ok(())
    }

    /// Calculate risk level using ISO 14971 risk matrix (Severity × Probability)
    fn calculate_risk_level(&self, severity: RiskSeverity, probability: RiskProbability) -> u8 {
        (severity as u8) * (probability as u8)
    }

    /// Determine risk acceptability based on risk level
    fn determine_acceptability(&self, risk_level: u8) -> RiskAcceptability {
        match risk_level {
            1..=5 => RiskAcceptability::Acceptable,
            6..=15 => RiskAcceptability::Tolerable,
            16..=25 => RiskAcceptability::Unacceptable,
            _ => RiskAcceptability::Unacceptable,
        }
    }

    /// Generate risk management report
    pub async fn generate_risk_report(
        &self,
        assessments: &[RiskAssessment],
        generated_by: String,
    ) -> Result<RiskManagementReport> {
        let total_assessments = assessments.len();
        let mut risk_level_distribution = HashMap::new();
        let mut acceptability_distribution = HashMap::new();
        let mut pending_control_measures = 0;

        for assessment in assessments {
            // Count risk levels
            let level_key = format!("Level_{}", assessment.initial_risk_level);
            *risk_level_distribution.entry(level_key).or_insert(0) += 1;

            // Count acceptability
            let acceptability_key = format!("{:?}", assessment.acceptability);
            *acceptability_distribution.entry(acceptability_key).or_insert(0) += 1;

            // Count pending control measures
            pending_control_measures += assessment.control_measures.iter()
                .filter(|cm| cm.verification_status == VerificationStatus::Pending)
                .count();
        }

        let report = RiskManagementReport {
            id: Uuid::new_v4(),
            generated_at: Utc::now(),
            generated_by: generated_by.clone(),
            total_assessments,
            risk_level_distribution,
            acceptability_distribution,
            pending_control_measures,
            compliance_status: self.assess_compliance_status(assessments),
        };

        // Log audit event
        self.audit_logger.log_event(
            &generated_by,
            "GENERATE_RISK_REPORT",
            &format!("risk_report:{}", report.id),
            "SUCCESS",
            Some(format!("Generated risk management report with {} assessments", total_assessments)),
        ).await?;

        Ok(report)
    }

    /// Assess overall compliance status
    fn assess_compliance_status(&self, assessments: &[RiskAssessment]) -> ComplianceStatus {
        let unacceptable_without_controls = assessments.iter()
            .any(|a| a.acceptability == RiskAcceptability::Unacceptable && a.control_measures.is_empty());

        let unverified_controls = assessments.iter()
            .any(|a| a.control_measures.iter()
                .any(|cm| cm.verification_status != VerificationStatus::Verified));

        if unacceptable_without_controls {
            ComplianceStatus::NonCompliant
        } else if unverified_controls {
            ComplianceStatus::RequiresAttention
        } else {
            ComplianceStatus::Compliant
        }
    }
}

/// Risk Management Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementReport {
    pub id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub total_assessments: usize,
    pub risk_level_distribution: HashMap<String, usize>,
    pub acceptability_distribution: HashMap<String, usize>,
    pub pending_control_measures: usize,
    pub compliance_status: ComplianceStatus,
}

/// Overall compliance status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    RequiresAttention,
    NonCompliant,
}

impl RiskSeverity {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(RiskSeverity::Negligible),
            2 => Ok(RiskSeverity::Minor),
            3 => Ok(RiskSeverity::Serious),
            4 => Ok(RiskSeverity::Critical),
            5 => Ok(RiskSeverity::Catastrophic),
            _ => Err(QmsError::Validation {
                field: "severity".to_string(),
                message: format!("Invalid risk severity value: {}. Must be 1-5", value),
            }),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RiskSeverity::Negligible => "Negligible harm",
            RiskSeverity::Minor => "Minor harm or injury",
            RiskSeverity::Serious => "Serious injury",
            RiskSeverity::Critical => "Severe injury or illness",
            RiskSeverity::Catastrophic => "Death or permanent disability",
        }
    }
}

impl RiskProbability {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(RiskProbability::Remote),
            2 => Ok(RiskProbability::Unlikely),
            3 => Ok(RiskProbability::Possible),
            4 => Ok(RiskProbability::Probable),
            5 => Ok(RiskProbability::Frequent),
            _ => Err(QmsError::Validation {
                field: "probability".to_string(),
                message: format!("Invalid risk probability value: {}. Must be 1-5", value),
            }),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RiskProbability::Remote => "Very unlikely to occur",
            RiskProbability::Unlikely => "Unlikely to occur",
            RiskProbability::Possible => "Possible to occur",
            RiskProbability::Probable => "Likely to occur",
            RiskProbability::Frequent => "Very likely or certain to occur",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_risk_level_calculation() {
        let audit_logger = AuditLogger::new_test();
        let service = RiskManagementService::new(audit_logger);

        // Test various severity/probability combinations
        assert_eq!(service.calculate_risk_level(RiskSeverity::Negligible, RiskProbability::Remote), 1);
        assert_eq!(service.calculate_risk_level(RiskSeverity::Catastrophic, RiskProbability::Frequent), 25);
        assert_eq!(service.calculate_risk_level(RiskSeverity::Serious, RiskProbability::Possible), 9);
    }

    #[tokio::test]
    async fn test_risk_acceptability_determination() {
        let audit_logger = AuditLogger::new_test();
        let service = RiskManagementService::new(audit_logger);

        assert_eq!(service.determine_acceptability(1), RiskAcceptability::Acceptable);
        assert_eq!(service.determine_acceptability(5), RiskAcceptability::Acceptable);
        assert_eq!(service.determine_acceptability(10), RiskAcceptability::Tolerable);
        assert_eq!(service.determine_acceptability(20), RiskAcceptability::Unacceptable);
        assert_eq!(service.determine_acceptability(25), RiskAcceptability::Unacceptable);
    }

    #[tokio::test]
    async fn test_create_risk_assessment() {
        let audit_logger = AuditLogger::new_test();
        let service = RiskManagementService::new(audit_logger);

        let assessment = service.create_risk_assessment(
            "Test Device".to_string(),
            "Electrical shock".to_string(),
            "User contact with live parts".to_string(),
            "Device failure → live parts exposed → user contact".to_string(),
            "Electric shock injury".to_string(),
            RiskSeverity::Critical,
            RiskProbability::Unlikely,
            "test_user".to_string(),
        ).await.unwrap();

        assert_eq!(assessment.device_name, "Test Device");
        assert_eq!(assessment.initial_severity, RiskSeverity::Critical);
        assert_eq!(assessment.initial_probability, RiskProbability::Unlikely);
        assert_eq!(assessment.initial_risk_level, 8); // 4 * 2
        assert_eq!(assessment.acceptability, RiskAcceptability::Tolerable);
        assert_eq!(assessment.status, RiskAssessmentStatus::Draft);
    }

    #[tokio::test]
    async fn test_approval_validation() {
        let audit_logger = AuditLogger::new_test();
        let service = RiskManagementService::new(audit_logger);

        let mut assessment = service.create_risk_assessment(
            "Test Device".to_string(),
            "High risk hazard".to_string(),
            "Dangerous situation".to_string(),
            "Sequence leading to harm".to_string(),
            "Severe harm".to_string(),
            RiskSeverity::Catastrophic,
            RiskProbability::Frequent,
            "test_user".to_string(),
        ).await.unwrap();

        // Should fail approval for unacceptable risk without control measures
        let result = service.approve_risk_assessment(&mut assessment, "reviewer".to_string()).await;
        assert!(result.is_err());

        // Add and verify control measure
        let mut control_measure = service.add_control_measure(
            assessment.id,
            ControlMeasureType::InherentSafety,
            "Safety interlock".to_string(),
            "Hardware safety switch".to_string(),
            "Functional testing".to_string(),
            "implementer".to_string(),
        ).await.unwrap();

        service.verify_control_measure(&mut control_measure, "verifier".to_string(), true).await.unwrap();
        assessment.control_measures.push(control_measure);

        // Should now succeed
        let result = service.approve_risk_assessment(&mut assessment, "reviewer".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(assessment.status, RiskAssessmentStatus::Approved);
    }

    #[tokio::test]
    async fn test_compliance_status_assessment() {
        let audit_logger = AuditLogger::new_test();
        let service = RiskManagementService::new(audit_logger);

        // Test compliant scenario
        let compliant_assessments = vec![];
        assert_eq!(service.assess_compliance_status(&compliant_assessments), ComplianceStatus::Compliant);

        // Test non-compliant scenario (unacceptable risk without controls)
        let non_compliant_assessment = service.create_risk_assessment(
            "Device".to_string(),
            "Hazard".to_string(),
            "Situation".to_string(),
            "Sequence".to_string(),
            "Harm".to_string(),
            RiskSeverity::Catastrophic,
            RiskProbability::Frequent,
            "user".to_string(),
        ).await.unwrap();

        let non_compliant_assessments = vec![non_compliant_assessment];
        assert_eq!(service.assess_compliance_status(&non_compliant_assessments), ComplianceStatus::NonCompliant);
    }
}