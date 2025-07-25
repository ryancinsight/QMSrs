use crate::{database::Database, error::Result, supplier::{Supplier, SupplierStatus}};
use chrono::NaiveDate;
use rusqlite::params;
use uuid::Uuid;

/// Repository for `suppliers` table
pub struct SupplierRepository {
    db: Database,
}

impl SupplierRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn insert(&self, supplier: &Supplier) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "INSERT INTO suppliers (
                    id, name, contact_info, qualification_status, qualification_date,
                    qualification_expiry_date, approved_by, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    supplier.id.to_string(),
                    supplier.name,
                    supplier.contact_info,
                    format!("{:?}", supplier.status),
                    supplier.qualification_date.map(|d| d.to_string()),
                    supplier.qualification_expiry_date.map(|d| d.to_string()),
                    supplier.approved_by,
                    supplier.created_at.to_rfc3339(),
                    supplier.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn update(&self, supplier: &Supplier) -> Result<()> {
        self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE suppliers SET
                    name = ?2,
                    contact_info = ?3,
                    qualification_status = ?4,
                    qualification_date = ?5,
                    qualification_expiry_date = ?6,
                    approved_by = ?7,
                    updated_at = ?8
                 WHERE id = ?1",
                params![
                    supplier.id.to_string(),
                    supplier.name,
                    supplier.contact_info,
                    format!("{:?}", supplier.status),
                    supplier.qualification_date.map(|d| d.to_string()),
                    supplier.qualification_expiry_date.map(|d| d.to_string()),
                    supplier.approved_by,
                    supplier.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn fetch_by_id(&self, id: &Uuid) -> Result<Option<Supplier>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, contact_info, qualification_status, qualification_date,
                        qualification_expiry_date, approved_by, created_at, updated_at
                 FROM suppliers WHERE id = ?1",
            )?;
            let mut rows = stmt.query(params![id.to_string()])?;
            if let Some(row) = rows.next()? {
                Ok(Some(self.row_to_supplier(row)?))
            } else {
                Ok(None)
            }
        })
    }

    fn row_to_supplier(&self, row: &rusqlite::Row) -> rusqlite::Result<Supplier> {
        let status_str: String = row.get(3)?;
        let status = match status_str.as_str() {
            "Pending" => SupplierStatus::Pending,
            "Qualified" => SupplierStatus::Qualified,
            "Disqualified" => SupplierStatus::Disqualified,
            _ => SupplierStatus::Pending,
        };
        Ok(Supplier {
            id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap(),
            name: row.get(1)?,
            contact_info: row.get(2)?,
            status,
            qualification_date: {
                let opt: Option<String> = row.get(4)?;
                opt.map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap())
            },
            qualification_expiry_date: {
                let opt: Option<String> = row.get(5)?;
                opt.map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap())
            },
            approved_by: row.get(6)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .unwrap()
                .with_timezone(&chrono::Utc),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    fn setup_repo() -> SupplierRepository {
        let db = Database::new(DatabaseConfig::default()).unwrap();
        SupplierRepository::new(db)
    }

    #[test]
    fn test_insert_and_fetch_supplier() {
        let repo = setup_repo();
        let supplier = Supplier {
            id: Uuid::new_v4(),
            name: "VendorX".to_string(),
            contact_info: Some("vendor@example.com".to_string()),
            status: SupplierStatus::Pending,
            qualification_date: None,
            qualification_expiry_date: None,
            approved_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        repo.insert(&supplier).unwrap();
        let fetched = repo.fetch_by_id(&supplier.id).unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, supplier.name);
    }
}