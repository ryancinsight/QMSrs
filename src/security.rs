use crate::{Result, QmsError, config::SecurityConfig};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, Key};
use aes_gcm::aead::{Aead, OsRng as AeadOsRng};
use ring::digest::{Context, SHA256};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Security manager for FDA-compliant operations
pub struct SecurityManager {
    config: SecurityConfig,
    active_sessions: HashMap<String, Session>,
    failed_attempts: HashMap<String, Vec<DateTime<Utc>>>,
}

impl SecurityManager {
    /// Create new security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            active_sessions: HashMap::new(),
            failed_attempts: HashMap::new(),
        }
    }

    /// Hash password using Argon2
    pub fn hash_password(&self, password: &str) -> Result<(String, String)> {
        if !self.validate_password_complexity(password) {
            return Err(QmsError::Security {
                message: "Password does not meet complexity requirements".to_string(),
            });
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| QmsError::Security {
                message: format!("Failed to hash password: {}", e),
            })?;

        Ok((password_hash.to_string(), salt.to_string()))
    }

    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| QmsError::Security {
                message: format!("Invalid password hash: {}", e),
            })?;

        let argon2 = Argon2::default();
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Validate password complexity
    pub fn validate_password_complexity(&self, password: &str) -> bool {
        if !self.config.password_complexity {
            return password.len() >= self.config.min_password_length as usize;
        }

        // FDA-compliant password requirements
        let has_length = password.len() >= self.config.min_password_length as usize;
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        has_length && has_upper && has_lower && has_digit && has_special
    }

    /// Check if user is locked out
    pub fn is_user_locked(&mut self, username: &str) -> bool {
        if let Some(attempts) = self.failed_attempts.get(username) {
            if attempts.len() >= self.config.max_login_attempts as usize {
                let last_attempt = attempts.last().unwrap();
                let lockout_duration = Duration::minutes(self.config.lockout_duration_minutes as i64);
                return Utc::now() < *last_attempt + lockout_duration;
            }
        }
        false
    }

    /// Record failed login attempt
    pub fn record_failed_attempt(&mut self, username: &str) {
        let now = Utc::now();
        self.failed_attempts
            .entry(username.to_string())
            .or_insert_with(Vec::new)
            .push(now);

        // Clean old attempts beyond lockout window
        let cutoff = now - Duration::minutes(self.config.lockout_duration_minutes as i64);
        if let Some(attempts) = self.failed_attempts.get_mut(username) {
            attempts.retain(|&attempt| attempt > cutoff);
        }
    }

    /// Clear failed attempts on successful login
    pub fn clear_failed_attempts(&mut self, username: &str) {
        self.failed_attempts.remove(username);
    }

    /// Create new session
    pub fn create_session(&mut self, user_id: String, ip_address: Option<String>) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::minutes(self.config.session_timeout_minutes as i64);

        let session = Session {
            id: session_id.clone(),
            user_id,
            ip_address,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            expires_at,
            is_active: true,
        };

        self.active_sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Validate session
    pub fn validate_session(&mut self, session_id: &str) -> Result<Option<&Session>> {
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            if session.is_active && Utc::now() < session.expires_at {
                session.last_activity = Utc::now();
                return Ok(Some(session));
            } else {
                session.is_active = false;
            }
        }
        Ok(None)
    }

    /// Revoke session
    pub fn revoke_session(&mut self, session_id: &str) -> Result<()> {
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.is_active = false;
        }
        Ok(())
    }

    /// Clean expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let now = Utc::now();
        self.active_sessions.retain(|_, session| {
            session.is_active && session.expires_at > now
        });
    }

    /// Encrypt sensitive data
    pub fn encrypt_data(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(QmsError::Encryption {
                message: "Key must be 32 bytes for AES-256".to_string(),
            });
        }

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| QmsError::Encryption {
                message: format!("Encryption failed: {}", e),
            })?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt sensitive data
    pub fn decrypt_data(&self, encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(QmsError::Encryption {
                message: "Key must be 32 bytes for AES-256".to_string(),
            });
        }

        if encrypted_data.len() < 12 {
            return Err(QmsError::Encryption {
                message: "Encrypted data too short".to_string(),
            });
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| QmsError::Encryption {
                message: format!("Decryption failed: {}", e),
            })
    }

    /// Generate cryptographic hash for integrity verification
    pub fn calculate_hash(&self, data: &[u8]) -> String {
        let mut context = Context::new(&SHA256);
        context.update(data);
        let digest = context.finish();
        hex::encode(digest.as_ref())
    }

    /// Generate digital signature for audit trail
    pub fn generate_signature(&self, data: &[u8], key: &[u8]) -> Result<String> {
        // Simplified signature - in production, use proper digital signatures
        let mut combined = Vec::new();
        combined.extend_from_slice(data);
        combined.extend_from_slice(key);
        Ok(self.calculate_hash(&combined))
    }
}

/// User session structure
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
}

/// User role for access control
#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    SystemAdmin,
    QualityManager,
    DocumentController,
    Auditor,
    User,
    ReadOnly,
}

impl UserRole {
    /// Check if role has permission for action
    pub fn has_permission(&self, permission: Permission) -> bool {
        match self {
            UserRole::SystemAdmin => true, // Admin has all permissions
            UserRole::QualityManager => matches!(permission,
                Permission::ReadDocuments | Permission::WriteDocuments | 
                Permission::ApproveDocuments | Permission::ViewAuditTrail |
                Permission::ManageUsers | Permission::CreateReports
            ),
            UserRole::DocumentController => matches!(permission,
                Permission::ReadDocuments | Permission::WriteDocuments |
                Permission::ApproveDocuments | Permission::ViewAuditTrail
            ),
            UserRole::Auditor => matches!(permission,
                Permission::ReadDocuments | Permission::ViewAuditTrail |
                Permission::CreateReports
            ),
            UserRole::User => matches!(permission,
                Permission::ReadDocuments | Permission::WriteDocuments
            ),
            UserRole::ReadOnly => matches!(permission,
                Permission::ReadDocuments
            ),
        }
    }
}

/// Permission enumeration for RBAC
#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    ReadDocuments,
    WriteDocuments,
    ApproveDocuments,
    ViewAuditTrail,
    ManageUsers,
    CreateReports,
    SystemAdmin,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_security_config() -> SecurityConfig {
        SecurityConfig {
            key_iterations: 100_000,
            session_timeout_minutes: 60,
            max_login_attempts: 3,
            lockout_duration_minutes: 30,
            password_complexity: true,
            min_password_length: 12,
        }
    }

    #[test]
    fn test_password_hashing_and_verification() {
        let security = SecurityManager::new(test_security_config());
        let password = "TestPass123!@#";
        
        let (hash, _salt) = security.hash_password(password).unwrap();
        assert!(security.verify_password(password, &hash).unwrap());
        assert!(!security.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_complexity_validation() {
        let security = SecurityManager::new(test_security_config());
        
        // Valid complex password
        assert!(security.validate_password_complexity("TestPass123!@#"));
        
        // Invalid passwords
        assert!(!security.validate_password_complexity("short"));
        assert!(!security.validate_password_complexity("nouppercase123!"));
        assert!(!security.validate_password_complexity("NOLOWERCASE123!"));
        assert!(!security.validate_password_complexity("NoNumbers!@#"));
        assert!(!security.validate_password_complexity("NoSpecialChars123"));
    }

    #[test]
    fn test_failed_login_attempts() {
        let mut security = SecurityManager::new(test_security_config());
        let username = "testuser";

        assert!(!security.is_user_locked(username));

        // Record failed attempts
        for _ in 0..3 {
            security.record_failed_attempt(username);
        }

        assert!(security.is_user_locked(username));

        // Clear attempts
        security.clear_failed_attempts(username);
        assert!(!security.is_user_locked(username));
    }

    #[test]
    fn test_session_management() {
        let mut security = SecurityManager::new(test_security_config());
        let user_id = "user123".to_string();
        let ip_address = Some("192.168.1.1".to_string());

        // Create session
        let session_id = security.create_session(user_id.clone(), ip_address).unwrap();
        
        // Validate session
        let session = security.validate_session(&session_id).unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().user_id, user_id);

        // Revoke session
        security.revoke_session(&session_id).unwrap();
        let session = security.validate_session(&session_id).unwrap();
        assert!(session.is_none());
    }

    #[test]
    fn test_encryption_decryption() {
        let security = SecurityManager::new(test_security_config());
        let key = b"12345678901234567890123456789012"; // 32 bytes
        let data = b"Sensitive FDA-regulated data";

        let encrypted = security.encrypt_data(data, key).unwrap();
        let decrypted = security.decrypt_data(&encrypted, key).unwrap();

        assert_eq!(data, &decrypted[..]);
    }

    #[test]
    fn test_user_role_permissions() {
        let admin = UserRole::SystemAdmin;
        let user = UserRole::User;
        let readonly = UserRole::ReadOnly;

        assert!(admin.has_permission(Permission::SystemAdmin));
        assert!(admin.has_permission(Permission::WriteDocuments));
        
        assert!(user.has_permission(Permission::ReadDocuments));
        assert!(user.has_permission(Permission::WriteDocuments));
        assert!(!user.has_permission(Permission::ApproveDocuments));
        
        assert!(readonly.has_permission(Permission::ReadDocuments));
        assert!(!readonly.has_permission(Permission::WriteDocuments));
    }

    #[test]
    fn test_hash_calculation() {
        let security = SecurityManager::new(test_security_config());
        let data = b"test data for hashing";
        
        let hash1 = security.calculate_hash(data);
        let hash2 = security.calculate_hash(data);
        
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
}