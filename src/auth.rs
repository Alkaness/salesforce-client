//! OAuth 2.0 authentication module
//!
//! Handles OAuth flows, token refresh, and credential management.

use crate::error::SfError;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// OAuth 2.0 credentials for Salesforce
#[derive(Debug, Clone)]
pub struct OAuthCredentials {
    /// OAuth client ID (Consumer Key)
    pub client_id: String,

    /// OAuth client secret (Consumer Secret)
    pub client_secret: String,

    /// Refresh token for obtaining new access tokens
    pub refresh_token: Option<String>,

    /// Username for password flow
    pub username: Option<String>,

    /// Password + security token for password flow
    pub password: Option<String>,
}

/// Response from OAuth token endpoint
#[derive(Debug, Deserialize, Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    instance_url: String,

    #[serde(default)]
    expires_in: Option<i64>,

    token_type: String,

    #[serde(default)]
    issued_at: Option<String>,
}

/// Managed access token with automatic refresh
#[derive(Debug, Clone)]
pub struct AccessToken {
    token: String,
    expires_at: Option<DateTime<Utc>>,
    instance_url: String,
}

impl AccessToken {
    /// Create a new access token
    pub fn new(token: String, instance_url: String, expires_in: Option<i64>) -> Self {
        let expires_at = expires_in.map(|secs| Utc::now() + Duration::seconds(secs));

        Self {
            token,
            expires_at,
            instance_url,
        }
    }

    /// Check if token is expired or about to expire (within 5 minutes)
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let buffer = Duration::minutes(5);
            Utc::now() + buffer >= expires_at
        } else {
            false // If no expiry, assume valid
        }
    }

    /// Get the token value
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the instance URL
    pub fn instance_url(&self) -> &str {
        &self.instance_url
    }
}

/// Token manager that handles automatic refresh
pub struct TokenManager {
    credentials: OAuthCredentials,
    current_token: Arc<RwLock<Option<AccessToken>>>,
    http_client: reqwest::Client,
    auth_url: String,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(credentials: OAuthCredentials) -> Self {
        Self {
            credentials,
            current_token: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::new(),
            auth_url: "https://login.salesforce.com".to_string(),
        }
    }

    /// Create a token manager for sandbox environment
    pub fn sandbox(credentials: OAuthCredentials) -> Self {
        let mut manager = Self::new(credentials);
        manager.auth_url = "https://test.salesforce.com".to_string();
        manager
    }

    /// Get a valid access token, refreshing if necessary
    ///
    /// This method ensures you always have a valid token by:
    /// 1. Checking if current token exists and is valid
    /// 2. If expired, automatically refreshing
    /// 3. Thread-safe access via RwLock
    pub async fn get_token(&self) -> Result<AccessToken, SfError> {
        // Fast path: check if token is valid (read lock)
        {
            let token_guard = self.current_token.read().await;
            if let Some(token) = token_guard.as_ref() {
                if !token.is_expired() {
                    debug!("Using cached access token");
                    return Ok(token.clone());
                }
            }
        }

        // Slow path: token expired or doesn't exist (write lock)
        info!("Access token expired or missing, refreshing...");
        let mut token_guard = self.current_token.write().await;

        // Double-check after acquiring write lock (another thread may have refreshed)
        if let Some(token) = token_guard.as_ref() {
            if !token.is_expired() {
                return Ok(token.clone());
            }
        }

        // Actually refresh the token
        let new_token = self.fetch_new_token().await?;
        *token_guard = Some(new_token.clone());

        info!("Successfully refreshed access token");
        Ok(new_token)
    }

    /// Fetch a new token from Salesforce
    async fn fetch_new_token(&self) -> Result<AccessToken, SfError> {
        // Try refresh token flow first
        if let Some(refresh_token) = &self.credentials.refresh_token {
            match self.refresh_token_flow(refresh_token).await {
                Ok(token) => return Ok(token),
                Err(e) => {
                    warn!(
                        "Refresh token flow failed: {}, falling back to password flow",
                        e
                    );
                }
            }
        }

        // Fall back to password flow
        if self.credentials.username.is_some() && self.credentials.password.is_some() {
            return self.password_flow().await;
        }

        Err(SfError::Auth(
            "No valid authentication method available".to_string(),
        ))
    }

    /// OAuth 2.0 Refresh Token Flow
    async fn refresh_token_flow(&self, refresh_token: &str) -> Result<AccessToken, SfError> {
        let url = format!("{}/services/oauth2/token", self.auth_url);

        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", &self.credentials.client_id),
            ("client_secret", &self.credentials.client_secret),
            ("refresh_token", refresh_token),
        ];

        let response = self.http_client.post(&url).form(&params).send().await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            return Err(SfError::Auth(format!("Token refresh failed: {}", body)));
        }

        let token_response: TokenResponse = response.json().await?;

        Ok(AccessToken::new(
            token_response.access_token,
            token_response.instance_url,
            token_response.expires_in,
        ))
    }

    /// OAuth 2.0 Password Flow (less secure, use for development only)
    async fn password_flow(&self) -> Result<AccessToken, SfError> {
        let username = self
            .credentials
            .username
            .as_ref()
            .ok_or_else(|| SfError::Auth("Username not provided".to_string()))?;
        let password = self
            .credentials
            .password
            .as_ref()
            .ok_or_else(|| SfError::Auth("Password not provided".to_string()))?;

        let url = format!("{}/services/oauth2/token", self.auth_url);

        let params = [
            ("grant_type", "password"),
            ("client_id", &self.credentials.client_id),
            ("client_secret", &self.credentials.client_secret),
            ("username", username),
            ("password", password),
        ];

        let response = self.http_client.post(&url).form(&params).send().await?;

        if !response.status().is_success() {
            let body = response.text().await?;
            return Err(SfError::Auth(format!("Authentication failed: {}", body)));
        }

        let token_response: TokenResponse = response.json().await?;

        Ok(AccessToken::new(
            token_response.access_token,
            token_response.instance_url,
            token_response.expires_in,
        ))
    }

    /// Invalidate the current token (force refresh on next request)
    pub async fn invalidate(&self) {
        let mut token_guard = self.current_token.write().await;
        *token_guard = None;
        info!("Access token invalidated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_token_expiry() {
        let token = AccessToken::new(
            "test_token".to_string(),
            "https://test.salesforce.com".to_string(),
            Some(3600), // 1 hour
        );

        assert!(!token.is_expired());
    }

    #[test]
    fn test_access_token_no_expiry() {
        let token = AccessToken::new(
            "test_token".to_string(),
            "https://test.salesforce.com".to_string(),
            None,
        );

        assert!(!token.is_expired());
    }
}
