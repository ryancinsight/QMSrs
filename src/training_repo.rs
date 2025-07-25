use crate::{database::Database, error::Result, training::{TrainingRecord, TrainingStatus}};
use chrono::NaiveDate;
use rusqlite::params;
use uuid::Uuid;

/// Repository layer for `training_records` persistence.
///
/// Adheres to the Repository pattern (GRASP) and keeps data-access logic
/// isolated from domain services. All operations are transactional and
/// leverage the central `Database` abstraction to maintain ACiD
/// properties required by FDA 21 CFR Part 11.
pub struct TrainingRepository {
    db: Database,
}

impl TrainingRepository {
    /// Create a new repository instance.
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Insert a new training record.
    pub fn insert(&self, record: &TrainingRecord) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "INSERT INTO training_records (
                    id, employee_id, training_item, mandatory, assigned_by,
                    due_date, completion_date, status, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    record.id.to_string(),
                    record.employee_id,
                    record.training_item,
                    record.mandatory as i32,
                    record.assigned_by,
                    record.due_date.to_string(),
                    record.completion_date.map(|d| d.to_string()),
                    format!("{:?}", record.status),
                    record.created_at.to_rfc3339(),
                    record.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    /// Update an existing training record.
    pub fn update(&self, record: &TrainingRecord) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE training_records SET
                    employee_id = ?2,
                    training_item = ?3,
                    mandatory = ?4,
                    assigned_by = ?5,
                    due_date = ?6,
                    completion_date = ?7,
                    status = ?8,
                    updated_at = ?9
                 WHERE id = ?1",
                params![
                    record.id.to_string(),
                    record.employee_id,
                    record.training_item,
                    record.mandatory as i32,
                    record.assigned_by,
                    record.due_date.to_string(),
                    record.completion_date.map(|d| d.to_string()),
                    format!("{:?}", record.status),
                    record.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    /// Fetch a single training record by ID.
    pub fn fetch_by_id(&self, id: &Uuid) -> Result<Option<TrainingRecord>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, employee_id, training_item, mandatory, assigned_by,
                        due_date, completion_date, status, created_at, updated_at
                 FROM training_records WHERE id = ?1",
            )?;

            let mut rows = stmt.query(params![id.to_string()])?;
            if let Some(row) = rows.next()? {
                Ok(Some(self.row_to_record(row)?))
            } else {
                Ok(None)
            }
        })
    }

    /// Fetch all training records for an employee.
    pub fn fetch_by_employee(&self, employee_id: &str) -> Result<Vec<TrainingRecord>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, employee_id, training_item, mandatory, assigned_by,
                        due_date, completion_date, status, created_at, updated_at
                 FROM training_records WHERE employee_id = ?1",
            )?;

            let record_iter = stmt.query_map(params![employee_id], |row| self.row_to_record(row))?;
            let mut records = Vec::new();
            for rec in record_iter {
                records.push(rec?);
            }
            Ok(records)
        })
    }

    /// Convert a rusqlite row into a `TrainingRecord` domain entity.
    fn row_to_record(&self, row: &rusqlite::Row) -> rusqlite::Result<TrainingRecord> {
        let status_str: String = row.get(7)?;
        let status = match status_str.as_str() {
            "Pending" => TrainingStatus::Pending,
            "InProgress" => TrainingStatus::InProgress,
            "Completed" => TrainingStatus::Completed,
            "Overdue" => TrainingStatus::Overdue,
            _ => TrainingStatus::Pending,
        };

        Ok(TrainingRecord {
            id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
            employee_id: row.get(1)?,
            training_item: row.get(2)?,
            mandatory: row.get::<_, i32>(3)? != 0,
            assigned_by: row.get(4)?,
            due_date: NaiveDate::parse_from_str(&row.get::<_, String>(5)?, "%Y-%m-%d").unwrap(),
            completion_date: {
                let opt: Option<String> = row.get(6)?;
                opt.map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap())
            },
            status,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                .unwrap()
                .with_timezone(&chrono::Utc),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::DatabaseConfig, audit::AuditLogger};

    fn setup_repo() -> TrainingRepository {
        let db = Database::new(DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false,
            backup_interval_hours: 24,
            backup_retention_days: 1,
        })
        .unwrap();
        TrainingRepository::new(db)
    }

    #[test]
    fn test_insert_and_fetch() {
        let repo = setup_repo();
        let record = TrainingRecord {
            id: Uuid::new_v4(),
            employee_id: "emp_test".to_string(),
            training_item: "Quality Overview".to_string(),
            mandatory: true,
            assigned_by: "manager".to_string(),
            due_date: chrono::Utc::now().date_naive(),
            completion_date: None,
            status: TrainingStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        repo.insert(&record).unwrap();
        let fetched = repo.fetch_by_id(&record.id).unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.training_item, record.training_item);
    }

    #[test]
    fn test_update_status() {
        let repo = setup_repo();
        let mut record = TrainingRecord {
            id: Uuid::new_v4(),
            employee_id: "emp_test".to_string(),
            training_item: "CAPA Training".to_string(),
            mandatory: false,
            assigned_by: "manager".to_string(),
            due_date: chrono::Utc::now().date_naive(),
            completion_date: None,
            status: TrainingStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        repo.insert(&record).unwrap();
        record.status = TrainingStatus::Completed;
        record.completion_date = Some(chrono::Utc::now().date_naive());
        record.updated_at = chrono::Utc::now();
        repo.update(&record).unwrap();

        let rec_db = repo.fetch_by_id(&record.id).unwrap().unwrap();
        assert_eq!(rec_db.status, TrainingStatus::Completed);
        assert!(rec_db.completion_date.is_some());
    }
}