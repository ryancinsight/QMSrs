//! RESTful API module for QMSrs
//! 
//! Phase 3 Objective: Provide external integration layer exposing key metrics
//! through a JSON-based API, enabling dashboards and third-party systems to
//! retrieve CAPA and Risk summaries.
//!
//! Design Principles:
//! - SOLID: `ApiState` has single responsibility of holding runtime state.
//! - CLEAN: No business logic leaks; aggregation delegates to existing services.
//! - FIRST: Tests are fast & isolated using in-memory state.
//! - INVEST: Self-contained feature deployable independently.

use std::sync::{Arc, RwLock};

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

use crate::capa::{CapaMetrics, CapaRecord, CapaService};
use crate::risk::{RiskAssessment, RiskManagementReport, RiskManagementService};
use crate::audit::{AuditLogger, AuditManager};
use crate::config::DatabaseConfig;
use crate::database::Database;

/// Shared application state for the API layer.
#[derive(Clone)]
pub struct ApiState {
    /// CAPA workflow service (includes audit integration)
    pub capa_service: CapaService,
    /// Risk management service (ISO 14971)
    pub risk_service: RiskManagementService,
    /// In-memory CAPA records used for aggregation
    pub capa_records: Arc<RwLock<Vec<CapaRecord>>>,
    /// In-memory risk assessments used for aggregation
    pub risk_assessments: Arc<RwLock<Vec<RiskAssessment>>>,
}

impl ApiState {
    /// Build a new API state backed by an isolated in-memory database. This keeps
    /// the API fully self-contained while exercising the real service layers.
    pub fn new() -> Self {
        // Configure in-memory SQLite database (shared cache disabled for safety)
        let db_config = DatabaseConfig {
            url: ":memory:".to_string(),
            max_connections: 10,
            wal_mode: false,
            backup_interval_hours: 24,
            backup_retention_days: 90,
        };
        let database = Database::new(db_config).expect("failed to init in-memory DB");
        let audit_manager = AuditManager::new(database);
        let capa_service = CapaService::new(audit_manager);

        // Risk service relies only on a lightweight audit logger
        let audit_logger = AuditLogger::new_test();
        let risk_service = RiskManagementService::new(audit_logger);

        Self {
            capa_service,
            risk_service,
            capa_records: Arc::new(RwLock::new(Vec::new())),
            risk_assessments: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

/// API response payload containing aggregated metrics.
#[derive(Serialize)]
pub struct MetricsResponse {
    /// Aggregated CAPA statistics
    pub capa_metrics: CapaMetrics,
    /// Aggregated Risk-management statistics
    pub risk_report: RiskManagementReport,
}

/// Handler for `GET /metrics`.
async fn get_metrics(State(state): State<ApiState>) -> impl IntoResponse {
    // Gather a snapshot of data under read locks to ensure consistency.
    let capa_records = state.capa_records.read().unwrap().clone();
    let risk_assessments = state.risk_assessments.read().unwrap().clone();

    // Compute metrics via domain services (SOLID adherence)
    let capa_metrics = state.capa_service.get_capa_metrics(&capa_records);
    let risk_report = match state
        .risk_service
        .generate_risk_report(&risk_assessments, "api_user".to_string())
        .await
    {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("risk report generation failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    (StatusCode::OK, Json(MetricsResponse { capa_metrics, risk_report })).into_response()
}

/// Build an Axum router with all API routes registered.
pub fn router() -> Router {
    let state = ApiState::new();
    Router::new()
        .route("/metrics", get(get_metrics))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Request};
    use hyper::Body;
    use tower::ServiceExt; // for `oneshot`
    use chrono::Utc;
    use crate::capa::{CapaPriority, CapaStatus, CapaType};
    use crate::risk::{RiskSeverity, RiskProbability};

    /// Build a router and underlying state for test purposes (FIRST compliant).
    async fn setup_test_router() -> (Router, ApiState) {
        let state = ApiState::new();
        let router = Router::new()
            .route("/metrics", get(get_metrics))
            .with_state(state.clone());
        (router, state)
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        // Arrange
        let (router, state) = setup_test_router().await;

        // Create sample CAPA record
        let mut capa = state
            .capa_service
            .create_capa(
                "Test CAPA".to_string(),
                "Test description".to_string(),
                CapaType::Preventive,
                CapaPriority::Medium,
                "initiator1".to_string(),
                "assignee1".to_string(),
                None,
            )
            .expect("create_capa failed");
        // Transition status to Closed for metrics diversity
        state
            .capa_service
            .update_status(&mut capa, CapaStatus::Closed, "initiator1", None)
            .expect("status update failed");
        state.capa_records.write().unwrap().push(capa);

        // Create sample Risk assessment
        let assessment = state
            .risk_service
            .create_risk_assessment(
                "Device X".to_string(),
                "Hazard description".to_string(),
                "Situation".to_string(),
                "Sequence".to_string(),
                "Harm description".to_string(),
                RiskSeverity::Minor,
                RiskProbability::Possible,
                "creator".to_string(),
            )
            .await
            .expect("risk assessment creation failed");
        state.risk_assessments.write().unwrap().push(assessment);

        // Act
        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed: MetricsResponse = serde_json::from_slice(&body).expect("valid JSON");
        assert_eq!(parsed.capa_metrics.total_count, 1);
        assert_eq!(parsed.risk_report.total_assessments, 1);
    }
}