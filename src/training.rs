//! # Training Records Module - ISO 13485 Compliance
//!
//! This module provides employee training and competency tracking to satisfy
//! FDA 21 CFR Part 820.25 and ISO 13485 \(Clause 6.2\) requirements.
//!
//! Design principles:
//! - SOLID: `TrainingService` responsible for training logic only.
//! - CLEAN & CUPID: Clear domain types, no external IO beyond audit logging.
//! - FIRST: Included tests are fast, isolated, repeatable.
//!
//! ## Key Features (Phase 3 scope)
//! * Create training records with due-dates & mandatory flag.
//! * Mark trainings complete with competency verification.
//! * Generate training metrics for dashboards & audits.

use crate::{audit::AuditLogger, error::Result};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::training_repo::TrainingRepository;

/// Training status lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainingStatus {
    Pending,
    InProgress,
    Completed,
    Overdue,
}

/// Employee training record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub id: Uuid,
    pub employee_id: String,
    pub training_item: String,
    pub mandatory: bool,
    pub assigned_by: String,
    pub due_date: NaiveDate,
    pub completion_date: Option<NaiveDate>,
    pub status: TrainingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TrainingRecord {
    /// Check and update status based on dates.
    fn refresh_status(&mut self) {
        if self.status == TrainingStatus::Completed {
            return;
        }
        let today = Utc::now().date_naive();
        if today > self.due_date {
            self.status = TrainingStatus::Overdue;
        }
    }
}

/// Aggregated metrics for dashboard/reporting
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrainingMetrics {
    pub total_count: usize,
    pub completed: usize,
    pub pending: usize,
    pub overdue: usize,
}

/// Service layer for training management
pub struct TrainingService {
    audit_logger: AuditLogger,
    repository: TrainingRepository,
}

impl TrainingService {
    pub fn new(audit_logger: AuditLogger, repository: TrainingRepository) -> Self {
        Self {
            audit_logger,
            repository,
        }
    }

    /// Assign a new training to employee
    pub async fn create_training_record(
        &self,
        employee_id: String,
        training_item: String,
        mandatory: bool,
        due_date: NaiveDate,
        assigned_by: String,
    ) -> Result<TrainingRecord> {
        let record = TrainingRecord {
            id: Uuid::new_v4(),
            employee_id: employee_id.clone(),
            training_item: training_item.clone(),
            mandatory,
            assigned_by: assigned_by.clone(),
            due_date,
            completion_date: None,
            status: TrainingStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Persist to database
        self.repository.insert(&record)?;

        // Audit log
        self.audit_logger
            .log_event(
                &assigned_by,
                "ASSIGN_TRAINING",
                &format!("training:{}", record.id),
                "SUCCESS",
                Some(format!(
                    "Assigned '{}' training to employee {} (mandatory={})",
                    training_item, employee_id, mandatory
                )),
            )
            .await?;

        Ok(record)
    }

    /// Mark training as completed with competency verification flag
    pub async fn mark_completed(
        &self,
        record: &mut TrainingRecord,
        completed_by: String,
        competency_verified: bool,
    ) -> Result<()> {
        record.completion_date = Some(Utc::now().date_naive());
        record.status = TrainingStatus::Completed;
        record.updated_at = Utc::now();

        // Persist update first
        self.repository.update(record)?;

        // Audit
        self.audit_logger
            .log_event(
                &completed_by,
                "COMPLETE_TRAINING",
                &format!("training:{}", record.id),
                "SUCCESS",
                Some(format!("competency_verified={}", competency_verified)),
            )
            .await?;
        Ok(())
    }

    /// Compute high-level metrics from records slice
    pub fn calculate_metrics(&self, records: &[TrainingRecord]) -> TrainingMetrics {
        let mut metrics = TrainingMetrics::default();
        metrics.total_count = records.len();
        for rec in records {
            match rec.status {
                TrainingStatus::Completed => metrics.completed += 1,
                TrainingStatus::Overdue => metrics.overdue += 1,
                _ => metrics.pending += 1,
            }
        }
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{audit::AuditLogger, config::DatabaseConfig};
    use crate::database::Database;
    use crate::training_repo::TrainingRepository;

    fn test_logger() -> AuditLogger {
        AuditLogger::new_test()
    }

    fn setup_service() -> TrainingService {
        let db = Database::new(DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false,
            backup_interval_hours: 24,
            backup_retention_days: 1,
        })
        .unwrap();
        let repo = TrainingRepository::new(db);
        TrainingService::new(test_logger(), repo)
    }

    #[tokio::test]
    async fn test_create_training_record() {
        let service = setup_service();
        let rec = service
            .create_training_record(
                "emp1".to_string(),
                "Quality System Overview".to_string(),
                true,
                Utc::now().date_naive(),
                "manager1".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(rec.status, TrainingStatus::Pending);
    }

    #[tokio::test]
    async fn test_mark_completed() {
        let service = setup_service();
        let mut rec = service
            .create_training_record(
                "emp1".to_string(),
                "Risk Management".to_string(),
                false,
                Utc::now().date_naive(),
                "manager1".to_string(),
            )
            .await
            .unwrap();
        service
            .mark_completed(&mut rec, "emp1".to_string(), true)
            .await
            .unwrap();
        assert_eq!(rec.status, TrainingStatus::Completed);
        assert!(rec.completion_date.is_some());
    }

    #[tokio::test]
    async fn test_metrics_calculation() {
        let service = setup_service();
        let mut records = Vec::new();
        // Completed
        let mut rec1 = service
            .create_training_record(
                "emp1".to_string(),
                "Doc Control".to_string(),
                true,
                Utc::now().date_naive(),
                "manager".to_string(),
            )
            .await
            .unwrap();
        service
            .mark_completed(&mut rec1, "emp1".to_string(), true)
            .await
            .unwrap();
        records.push(rec1);

        // Pending
        let rec2 = service
            .create_training_record(
                "emp2".to_string(),
                "CAPA Process".to_string(),
                true,
                Utc::now().date_naive(),
                "manager".to_string(),
            )
            .await
            .unwrap();
        records.push(rec2);

        // Overdue (set due date to yesterday)
        let rec3 = TrainingRecord {
            id: Uuid::new_v4(),
            employee_id: "emp3".to_string(),
            training_item: "Audit Trail".to_string(),
            mandatory: true,
            assigned_by: "manager".to_string(),
            due_date: (Utc::now() - chrono::Duration::days(1)).date_naive(),
            completion_date: None,
            status: TrainingStatus::Overdue,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        records.push(rec3);

        let metrics = service.calculate_metrics(&records);
        assert_eq!(metrics.total_count, 3);
        assert_eq!(metrics.completed, 1);
        assert_eq!(metrics.pending, 1);
        assert_eq!(metrics.overdue, 1);
    }
}