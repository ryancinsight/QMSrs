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
use std::net::SocketAddr;
use hyper::Error as HyperError;
use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use axum::middleware::{self, Next};
use axum::http::{Request, header::AUTHORIZATION};
use uuid::Uuid;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

use crate::capa::{CapaMetrics, CapaRecord, CapaService};
use crate::risk::{RiskAssessment, RiskManagementReport, RiskManagementService};
use crate::audit::{AuditLogger, AuditManager};
use crate::config::DatabaseConfig;
use crate::database::Database;
use crate::supplier::{Supplier, SupplierService, SupplierMetrics};

/// In-memory representation of an API token with TTL & scopes.
#[derive(Clone, Debug)]
pub struct ApiToken {
    /// Raw token string (opaque)
    pub token: String,
    /// Expiration timestamp (UTC)
    pub expires_at: DateTime<Utc>,
    /// Allowed scopes (e.g., "metrics:read")
    pub scopes: Vec<String>,
}

impl ApiToken {
    /// Check whether token is still valid and has required scope.
    pub fn is_valid(&self, scope: &str) -> bool {
        Utc::now() < self.expires_at && self.scopes.iter().any(|s| s == scope)
    }
}

/// Simple in-memory token manager – suitable for embedded API use cases.
#[derive(Clone, Debug, Default)]
pub struct TokenManager {
    tokens: Arc<RwLock<HashMap<String, ApiToken>>>,
}

impl TokenManager {
    /// Create a new token manager with zero tokens.
    pub fn new() -> Self {
        Self { tokens: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Insert a new token with TTL (minutes) and scopes.
    pub fn insert_token(&self, token: String, ttl_minutes: i64, scopes: Vec<String>) {
        let expires_at = Utc::now() + Duration::minutes(ttl_minutes);
        let api_token = ApiToken { token: token.clone(), expires_at, scopes };
        self.tokens.write().unwrap().insert(token, api_token);
    }

    /// Validate incoming token string for required scope.
    pub fn validate(&self, token: &str, scope: &str) -> bool {
        if let Some(stored) = self.tokens.read().unwrap().get(token) {
            stored.is_valid(scope)
        } else {
            false
        }
    }
}

/// Shared application state for the API layer.
#[derive(Clone)]
pub struct ApiState {
    /// CAPA workflow service (includes audit integration)
    pub capa_service: CapaService,
    /// Risk management service (ISO 14971)
    pub risk_service: RiskManagementService,
    /// Supplier management service
    pub supplier_service: SupplierService,
    /// In-memory CAPA records used for aggregation
    pub capa_records: Arc<RwLock<Vec<CapaRecord>>>,
    /// In-memory risk assessments used for aggregation
    pub risk_assessments: Arc<RwLock<Vec<RiskAssessment>>>,
    /// In-memory supplier records used for aggregation
    pub suppliers: Arc<RwLock<Vec<Supplier>>>,
    /// Token manager holding API auth tokens
    pub token_manager: TokenManager,
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
        let audit_manager = AuditManager::new(database.clone());
        let capa_service = CapaService::new(audit_manager);

        // Risk service relies only on a lightweight audit logger
        let risk_logger = AuditLogger::new_test();
        let risk_service = RiskManagementService::new(risk_logger);

        // Supplier service (separate logger for better isolation)
        let supplier_logger = AuditLogger::new_test();
        let supplier_repository = crate::supplier_repo::SupplierRepository::new(database.clone());
        let supplier_service = SupplierService::new(supplier_logger, supplier_repository);

        Self {
            capa_service,
            risk_service,
            supplier_service,
            capa_records: Arc::new(RwLock::new(Vec::new())),
            risk_assessments: Arc::new(RwLock::new(Vec::new())),
            suppliers: Arc::new(RwLock::new(Vec::new())),
            token_manager: TokenManager::new(),
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

/// Handler for `GET /supplier_metrics`.
async fn get_supplier_metrics(State(state): State<ApiState>) -> impl IntoResponse {
    let suppliers = state.suppliers.read().unwrap().clone();
    let metrics = SupplierMetrics::from_suppliers(&suppliers);
    (StatusCode::OK, Json(metrics)).into_response()
}

/// Middleware: Enforces Bearer token authentication and scope validation.
async fn token_auth<B>(
    State(state): State<ApiState>,
    req: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    const REQUIRED_SCOPE: &str = "metrics:read";

    // Extract token from `Authorization: Bearer <token>` header
    let unauthorized = || (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    let Some(header_val) = req.headers().get(AUTHORIZATION) else {
        return unauthorized();
    };
    let Ok(auth_str) = header_val.to_str() else {
        return unauthorized();
    };
    let token = auth_str.strip_prefix("Bearer ").unwrap_or("");

    if state.token_manager.validate(token, REQUIRED_SCOPE) {
        next.run(req).await
    } else {
        unauthorized()
    }
}

/// Build an Axum router with all API routes registered.
pub fn router() -> Router {
    let state = ApiState::new();

    // For demonstration, generate a default token valid for 24 hours with metrics scope.
    let default_token = Uuid::new_v4().to_string();
    state.token_manager.insert_token(default_token.clone(), 60 * 24, vec!["metrics:read".to_string()]);
    tracing::info!("API authentication token generated", %default_token);

    Router::new()
        .route("/metrics", get(get_metrics))
        .route("/supplier_metrics", get(get_supplier_metrics))
        .layer(middleware::from_fn_with_state(state.clone(), token_auth))
        .with_state(state)
}

pub use MetricsResponse;

/// Start the API server on the provided address (e.g., "127.0.0.1:3000").
/// This is intended to run in a background Tokio task.
pub async fn serve(addr: &str) -> Result<(), HyperError> {
    let socket: SocketAddr = addr.parse().expect("invalid socket address");
    let router = router();
    axum::Server::bind(&socket)
        .serve(router.into_make_service())
        .await
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
    use axum::http::header::{AUTHORIZATION, HeaderValue};
    use crate::supplier::{Supplier, SupplierStatus, SupplierMetrics};

    /// Build a router and underlying state for test purposes (FIRST compliant).
    async fn setup_test_router() -> (Router, ApiState) {
        let state = ApiState::new();
        let router = Router::new()
            .route("/metrics", get(super::get_metrics))
            .route("/supplier_metrics", get(super::get_supplier_metrics))
            .layer(middleware::from_fn_with_state(state.clone(), super::token_auth))
            .with_state(state.clone());
        (router, state)
    }

    /// Helper: obtain valid token from state after setup.
    async fn setup_test_router_with_token() -> (Router, String) {
        let (router, state) = setup_test_router().await;
        // Insert token valid for tests
        let token = "test-token".to_string();
        state.token_manager.insert_token(token.clone(), 60, vec!["metrics:read".to_string()]);
        (router, token)
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        // Arrange
        let (router, state) = setup_test_router().await;

        // Insert valid token for this test
        let token = "metrics-token".to_string();
        state.token_manager.insert_token(token.clone(), 60, vec!["metrics:read".to_string()]);

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
                    .header(
                        AUTHORIZATION,
                        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
                    )
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

    #[tokio::test]
    async fn test_metrics_endpoint_requires_auth() {
        let (router, _token) = setup_test_router_with_token().await;

        // Request without token should be 401
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
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_metrics_endpoint_with_valid_token() {
        let (router, token) = setup_test_router_with_token().await;

        let auth_header = format!("Bearer {}", token);
        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/metrics")
                    .header(AUTHORIZATION, HeaderValue::from_str(&auth_header).unwrap())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_supplier_metrics_endpoint() {
        let (router, state) = setup_test_router().await;
        let token = "supplier-token".to_string();
        state.token_manager.insert_token(token.clone(), 60, vec!["metrics:read".to_string()]);

        // Add sample suppliers
        let mut suppliers_guard = state.suppliers.write().unwrap();
        use uuid::Uuid;
        suppliers_guard.extend(vec![
            Supplier {
                id: Uuid::new_v4(),
                name: "Vendor1".to_string(),
                contact_info: None,
                status: SupplierStatus::Qualified,
                qualification_date: None,
                qualification_expiry_date: None,
                approved_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Supplier {
                id: Uuid::new_v4(),
                name: "Vendor2".to_string(),
                contact_info: None,
                status: SupplierStatus::Pending,
                qualification_date: None,
                qualification_expiry_date: None,
                approved_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ]);
        drop(suppliers_guard);

        // Perform request
        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/supplier_metrics")
                    .header(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed: SupplierMetrics = serde_json::from_slice(&body).expect("valid JSON");
        assert_eq!(parsed.total_count, 2);
        assert_eq!(parsed.qualified_count, 1);
    }
}