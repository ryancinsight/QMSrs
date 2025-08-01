# Product Requirements Document (PRD) - FDA Compliant Medical Device QMS System

## 1. Product Overview
**Product Name**: QMSrs - FDA Compliant Medical Device Quality Management System
**Version**: 1.3.0
**Target**: FDA 21 CFR Part 820 compliance
**Platform**: Terminal-based application using Rust and ratatui

## 2. Requirements (INVEST Criteria)

### 2.1 Independent Requirements
- **REQ-001**: Document Control System - Manage controlled documents with version control, approval workflows ✅ COMPLETED
- **REQ-002**: Risk Management Module - ISO 14971 compliant risk assessment and mitigation tracking ✅ COMPLETED
- **REQ-003**: CAPA System - Corrective and Preventive Action workflow management ✅ COMPLETED
- **REQ-004**: Audit Trail - Complete audit logging of all system activities ✅ COMPLETED
- **REQ-005**: User Management - Role-based access control (RBAC) with FDA compliant user authentication ✅ COMPLETED

### 2.2 Negotiable Features
- **REQ-006**: Reporting Dashboard - Generate FDA-required reports and metrics
- **REQ-007**: Training Records - Track employee training and competency
- **REQ-008**: Supplier Management - Vendor qualification and monitoring

### 2.3 Valuable Outcomes
- **VAL-001**: Reduce FDA audit preparation time by 80%
- **VAL-002**: Ensure 100% traceability of quality records
- **VAL-003**: Automate compliance workflows
- **VAL-004**: Enable proactive risk identification and mitigation (NEW)
- **VAL-005**: Streamline CAPA investigation and closure processes (NEW)

### 2.4 Estimable Components
- **Core TUI Framework**: 2-3 days ✅ COMPLETED
- **Document Control**: 3-4 days ✅ COMPLETED
- **Risk Management**: 4-5 days ✅ COMPLETED
- **CAPA System**: 3-4 days
- **Audit Trail**: 2-3 days ✅ COMPLETED

### 2.5 Small, Testable Features
Each requirement broken into testable units with acceptance criteria

## 2.6 Phase 2 Specific Requirements (COMPLETED)

### 2.6.1 Risk Management Module (ISO 14971)
- **REQ-R001**: Risk Assessment Creation and Management
  - **I**: Independent from other modules, self-contained risk evaluation
  - **N**: Risk severity and probability matrices are configurable
  - **V**: Enables systematic risk identification per ISO 14971
  - **E**: 2-3 days for core assessment functionality
  - **S**: Single responsibility: risk evaluation and tracking
  - **T**: Testable with risk scenarios and calculation verification

- **REQ-R002**: Risk Control Measures Implementation
  - **I**: Independent risk mitigation tracking system
  - **N**: Control measure types and effectiveness metrics configurable
  - **V**: Provides systematic risk reduction capability
  - **E**: 1-2 days for control measure tracking
  - **S**: Focused on risk control and monitoring
  - **T**: Testable with control measure effectiveness validation

- **REQ-R003**: Risk Management Database Schema
  - **I**: Standalone risk data persistence layer
  - **N**: Schema extensible for different risk assessment methodologies
  - **V**: Enables long-term risk trend analysis
  - **E**: 1 day for schema design and implementation
  - **S**: Single purpose: risk data storage and retrieval
  - **T**: Testable with data integrity and query performance tests

### 2.6.2 CAPA System Implementation
- **REQ-C001**: CAPA Workflow Management
  - **I**: Independent workflow engine for CAPA processes
  - **N**: Workflow steps and approval processes configurable
  - **V**: Ensures systematic problem investigation and resolution
  - **E**: 2-3 days for workflow implementation
  - **S**: Single focus: CAPA lifecycle management
  - **T**: Testable with end-to-end CAPA scenarios

- **REQ-C002**: CAPA-Risk Integration
  - **I**: Standalone integration layer between CAPA and Risk modules
  - **N**: Integration depth and automation level configurable
  - **V**: Connects problem identification to risk mitigation
  - **E**: 1 day for integration implementation
  - **S**: Single purpose: linking CAPA actions to risk controls
  - **T**: Testable with integrated workflow scenarios

## 3. Technical Architecture

### 3.1 Technology Stack
- **Language**: Rust 1.70+
- **TUI Framework**: ratatui
- **Database**: SQLite with WAL mode
- **Encryption**: AES-256 for sensitive data
- **Logging**: tracing crate for audit trails

### 3.2 Compliance Standards
- **FDA 21 CFR Part 820**: Quality System Regulation
- **ISO 13485**: Medical Device Quality Management
- **ISO 14971**: Risk Management for Medical Devices
- **FDA 21 CFR Part 11**: Electronic Records and Signatures

### 3.3 Phase 2 Architecture Extensions
- **Risk Management Service**: Dedicated service layer for ISO 14971 compliance
- **CAPA Workflow Engine**: State machine for CAPA process management
- **Integration Layer**: Connects risk assessments with CAPA actions
- **Reporting Service**: Risk and CAPA metrics and compliance reports

## 4. Success Criteria
- **DONE Definition**: 100% test coverage, FDA compliance verified, documented, reviewed
- **Performance**: Sub-second response times for all operations
- **Security**: Encrypted data at rest and in transit
- **Usability**: Intuitive terminal interface with keyboard shortcuts
- **Phase 2 DONE**: Risk assessments comply with ISO 14971, CAPA workflows meet FDA requirements

## 5. Constraints
- **Regulatory**: Must maintain FDA audit trail requirements
- **Performance**: Real-time responsiveness required
- **Security**: Must support encrypted storage and access controls
- **Platform**: Cross-platform terminal compatibility
- **ISO 14971**: Risk management must follow medical device risk assessment standards

## 6. Release Plan
- **Phase 1**: Core TUI + Document Control + Audit Trail ✅ COMPLETED ✅
  - ✅ TUI Framework with ratatui integration (TASK-003, TASK-013)
  - ✅ Document Control System with FDA compliance (TASK-009, TASK-010)
  - ✅ Comprehensive Audit Trail system (TASK-007, TASK-008)
  - ✅ SQLite database with WAL mode (TASK-005, TASK-006)
  - ✅ AES-256 encryption for sensitive data (TASK-011)
  - ✅ User authentication and session management (TASK-012)
  - ✅ Main application integration with async runtime (TASK-013)
  - ✅ End-to-end TUI workflow testing (TASK-014)
- **Phase 2**: Risk Management + CAPA System ✅ COMPLETED
  - ✅ Risk Management Module (ISO 14971 compliance) - TASK-015
  - ✅ Risk Management Database Schema - TASK-016  
  - ✅ CAPA System (Corrective and Preventive Action workflow) - TASK-017
  - ✅ CAPA-TUI Integration - TASK-018
  - Advanced TUI features and user interaction
  - Performance optimization and scalability
- **Phase 3**: Reporting + User Management + Training Records ✅ COMPLETED
  - ✅ JSON Metrics Endpoint and Axum API service (TASK-020, TASK-021)
  - Reporting Dashboard with FDA-required reports
  - Training Records and competency tracking
  - Supplier Management and vendor qualification
- **Phase 4**: Performance Optimization & Compliance PDF Reporting ✅ COMPLETED
  - ✅ Response time <100 ms for API endpoints (TASK-031)
  - ✅ In-memory caching layer for metrics (TASK-031)
  - ✅ PDF export engine for compliance reports (TASK-033)
  - ✅ PDF template library and branding guidelines (TASK-032)
  - Further UX refinements and accessibility (TASK-034,TASK-035 ongoing)

- **Phase 5**: Post-Market Surveillance & Cloud Synchronization (NEXT)
  - ✅ Adverse Event Logging core domain & repository (TASK-040–041)
  - Cloud backup service scaffolding (TASK-042)
  - Real-time signal detection prototype (TASK-043)

## 7. Acceptance Criteria (Phase 2)

### Risk Management Module
- **AC-R001**: System shall create, edit, and delete risk assessments per ISO 14971
- **AC-R002**: Risk matrix calculations shall be accurate and auditable
- **AC-R003**: Risk control measures shall be tracked with effectiveness metrics
- **AC-R004**: All risk management activities shall be logged in audit trail
- **AC-R005**: Risk data shall be encrypted and access-controlled

### CAPA System
- **AC-C001**: CAPA workflow shall support investigation, root cause analysis, and closure
- **AC-C002**: CAPA actions shall integrate with risk control measures
- **AC-C003**: CAPA effectiveness verification shall be tracked and documented
- **AC-C004**: All CAPA activities shall maintain complete audit trail
- **AC-C005**: CAPA system shall support FDA inspection requirements

## 2.7 Phase 3 Specific Requirements (CURRENT FOCUS)

### 2.7.1 RESTful API Integration ✅ COMPLETED
- **REQ-API001**: JSON Metrics Endpoint exposing CAPA and Risk summaries ✅ COMPLETED

### 2.7.2 Advanced Reporting Dashboard ✅ COMPLETED
- **REQ-REP001**: TUI Reports Tab to fetch live metrics via API ✅ COMPLETED

### 2.7.3 External System Authentication ✅ COMPLETED
- **REQ-AUTH001**: API token-based authentication
  - **I**: Guards only API routes
  - **N**: Token TTL and scopes configurable
  - **V**: Secures external access channels
  - **E**: 1 day implementation (actual: 0.5 day)
  - **S**: Authentication/authorization responsibility
  - **T**: Tested with positive, missing, and invalid token scenarios – all passing

### 2.7.4 Training Records & Competency Tracking ✅ **COMPLETED**

- **REQ-TRAIN001**: Employee Training Record Management ✅ **COMPLETED**
  - **I**: Independent module for training data; no CAPA dependency
  - **N**: Training types (SOP, Policy, Safety) configurable
  - **V**: Ensures personnel competency evidence for audits
  - **E**: 2 days for core CRUD & metrics
  - **S**: Single responsibility: training management
  - **T**: Testable via training scenarios and metric calculations

- **REQ-TRAIN002**: Training Metrics Dashboard Integration ✅ **COMPLETED**
  - **I**: Aggregates data from training module only
  - **N**: KPI formulas adjustable (completion %, overdue count)
  - **V**: Provides at-a-glance compliance status
  - **E**: 1 day for metrics aggregation
  - **S**: Single purpose: KPI reporting
  - **T**: Testable via simulated datasets

### 2.7.5 Supplier Management & Qualification (COMPLETED)

- **REQ-SUP001**: Supplier Record Management ✅ COMPLETED
  - **I**: Independent module; no CAPA dependency
  - **N**: Supplier types and risk categories configurable
  - **V**: Enables compliant supplier qualification evidence
  - **E**: 2 days for core CRUD & metrics
  - **S**: Single responsibility: supplier management
  - **T**: Testable via supplier lifecycle scenarios

- **REQ-SUP002**: Supplier Metrics Dashboard Integration ✅ COMPLETED
  - **I**: Aggregates data from supplier module only
  - **N**: KPI formulas adjustable (qualified %, disqualified count)
  - **V**: Provides compliance insights on supply chain
  - **E**: 1 day for aggregation & display
  - **S**: Single purpose: KPI reporting
  - **T**: Testable via simulated datasets

- **REQ-SUP003**: Supplier TUI Integration ✅ COMPLETED

### Acceptance Criteria – Supplier Management
- **AC-SUP001**: System shall create, edit, qualify, and disqualify suppliers with full audit trail
- **AC-SUP002**: Supplier qualification status shall be traceable and encrypted
- **AC-SUP003**: Supplier metrics shall be accurate and available via API
- **AC-SUP004**: All supplier data actions shall satisfy FDA Part 11 signature/logging requirements