use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::database::Database;
use crate::error::{QmsError, Result};

/// Adverse event severity levels per FDA guidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Major,
    Minor,
}

/// Domain model representing an adverse event record.
#[derive(Debug, Clone)]
pub struct AdverseEvent {
    pub id: Uuid,
    pub reported_on: DateTime<Utc>,
    pub reporter: String,
    pub description: String,
    pub severity: Severity,
}

impl AdverseEvent {
    /// Factory method to create a new adverse event with current timestamp.
    pub fn new<S1: Into<String>, S2: Into<String>>(reporter: S1, description: S2, severity: Severity) -> Self {
        Self {
            id: Uuid::new_v4(),
            reported_on: Utc::now(),
            reporter: reporter.into(),
            description: description.into(),
            severity,
        }
    }
}

/// Repository handling persistence of adverse events.
pub struct AdverseEventRepo<'a> {
    db: &'a Database,
}

impl<'a> AdverseEventRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Persist a new adverse event entry.
    pub fn insert(&self, event: &AdverseEvent) -> Result<()> {
        let conn = self.db.get_conn()?;
        conn.execute(
            "INSERT INTO adverse_events (id, reported_on, reporter, description, severity)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                event.id.to_string(),
                event.reported_on.to_rfc3339(),
                &event.reporter,
                &event.description,
                event.severity as i32,
            ),
        )?;
        Ok(())
    }

    /// Fetch an event by UUID.
    pub fn get(&self, id: Uuid) -> Result<AdverseEvent> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, reported_on, reporter, description, severity FROM adverse_events WHERE id = ?1",
        )?;
        let row = stmt.query_row((id.to_string(),), |row| {
            Ok(AdverseEvent {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).map_err(|e| QmsError::Application { message: format!("Invalid UUID in DB: {e}") })?,
                reported_on: DateTime::parse_from_rfc3339(row.get::<_, String>(1)?.as_str())
                    .map_err(|e| QmsError::Application { message: format!("Invalid timestamp in DB: {e}") })?
                    .with_timezone(&Utc),
                reporter: row.get(2)?,
                description: row.get(3)?,
                severity: match row.get::<_, i32>(4)? {
                    0 => Severity::Critical,
                    1 => Severity::Major,
                    _ => Severity::Minor,
                },
            })
        })?;
        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    #[test]
    fn test_insert_and_get_event() {
        let db = Database::in_memory().unwrap();
        db.initialize_schema().unwrap();
        // add adverse_events table for tests
        {
            let conn = db.get_conn().unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS adverse_events (
                    id TEXT PRIMARY KEY,
                    reported_on TEXT NOT NULL,
                    reporter TEXT NOT NULL,
                    description TEXT NOT NULL,
                    severity INTEGER NOT NULL
                )",
                (),
            )
            .unwrap();
        }
        let repo = AdverseEventRepo::new(&db);
        let event = AdverseEvent::new("tester", "failure mode detected", Severity::Major);
        repo.insert(&event).unwrap();

        let fetched = repo.get(event.id).unwrap();
        assert_eq!(fetched.description, "failure mode detected");
        assert_eq!(fetched.severity, Severity::Major);
    }
}