//! CRUD operations (Create, Read, Update, Delete)
//!
//! Provides type-safe methods for manipulating Salesforce records.

use crate::error::{SfError, SfResult};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Response from a successful insert operation
#[derive(Debug, Deserialize, Clone)]
pub struct InsertResponse {
    /// The ID of the newly created record
    pub id: String,

    /// Whether the operation was successful
    pub success: bool,

    /// Any errors that occurred
    #[serde(default)]
    pub errors: Vec<SalesforceError>,
}

/// Response from an update or delete operation
#[derive(Debug, Deserialize, Clone)]
pub struct UpdateResponse {
    /// Whether the operation was successful
    pub success: bool,

    /// Any errors that occurred
    #[serde(default)]
    pub errors: Vec<SalesforceError>,
}

/// Salesforce error detail
#[derive(Debug, Deserialize, Clone)]
pub struct SalesforceError {
    /// Error status code
    #[serde(rename = "statusCode")]
    pub status_code: String,

    /// Error message
    pub message: String,

    /// Fields that caused the error
    #[serde(default)]
    pub fields: Vec<String>,
}

/// Batch response for multiple operations
#[derive(Debug, Deserialize)]
pub struct BatchResponse {
    /// Whether the batch operation succeeded
    #[serde(rename = "hasErrors")]
    pub has_errors: bool,

    /// Results for each record
    pub results: Vec<BatchResult>,
}

/// Result for a single record in a batch operation
#[derive(Debug, Deserialize)]
pub struct BatchResult {
    /// Status code
    #[serde(rename = "statusCode")]
    pub status_code: u16,

    /// Result details (ID for insert, empty for update/delete)
    pub result: Option<serde_json::Value>,
}

/// Builder for upsert operations
#[derive(Debug)]
pub struct UpsertBuilder {
    /// External ID field name
    pub external_id_field: String,

    /// External ID value
    pub external_id_value: String,
}

impl Clone for UpsertBuilder {
    fn clone(&self) -> Self {
        Self {
            external_id_field: self.external_id_field.clone(),
            external_id_value: self.external_id_value.clone(),
        }
    }
}

impl UpsertBuilder {
    /// Create a new upsert builder
    pub fn new(external_id_field: impl Into<String>, external_id_value: impl Into<String>) -> Self {
        Self {
            external_id_field: external_id_field.into(),
            external_id_value: external_id_value.into(),
        }
    }
}

/// CRUD operations implementation
pub(crate) struct CrudOperations {
    http_client: reqwest::Client,
    base_url: String,
    access_token: String,
}

impl CrudOperations {
    /// Create a new CRUD operations handler
    pub fn new(http_client: reqwest::Client, base_url: String, access_token: String) -> Self {
        Self {
            http_client,
            base_url,
            access_token,
        }
    }

    /// Insert a new record
    ///
    /// # Example
    /// ```ignore
    /// #[derive(Serialize)]
    /// struct NewAccount {
    ///     #[serde(rename = "Name")]
    ///     name: String,
    /// }
    ///
    /// let account = NewAccount { name: "Acme Corp".to_string() };
    /// let response = client.insert("Account", &account).await?;
    /// println!("Created account with ID: {}", response.id);
    /// ```
    pub async fn insert<T: Serialize>(&self, sobject: &str, data: &T) -> SfResult<InsertResponse> {
        let url = format!("{}/services/data/v57.0/sobjects/{}", self.base_url, sobject);

        debug!("Inserting {} record", sobject);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            return Err(SfError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let insert_response: InsertResponse = response.json().await?;

        if !insert_response.success {
            let error_msg = insert_response
                .errors
                .iter()
                .map(|e| format!("{}: {}", e.status_code, e.message))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(SfError::Api {
                status: 400,
                body: error_msg,
            });
        }

        info!(
            "Successfully inserted {} with ID: {}",
            sobject, insert_response.id
        );
        Ok(insert_response)
    }

    /// Update an existing record
    ///
    /// # Example
    /// ```ignore
    /// #[derive(Serialize)]
    /// struct AccountUpdate {
    ///     #[serde(rename = "Name")]
    ///     name: String,
    /// }
    ///
    /// let update = AccountUpdate { name: "New Name".to_string() };
    /// client.update("Account", "001xx000003DGbX", &update).await?;
    /// ```
    pub async fn update<T: Serialize>(&self, sobject: &str, id: &str, data: &T) -> SfResult<()> {
        let url = format!(
            "{}/services/data/v57.0/sobjects/{}/{}",
            self.base_url, sobject, id
        );

        debug!("Updating {} record {}", sobject, id);

        let response = self
            .http_client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .await?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(SfError::NotFound {
                sobject: sobject.to_string(),
                id: id.to_string(),
            });
        }

        if !status.is_success() {
            let body = response.text().await?;
            return Err(SfError::Api {
                status: status.as_u16(),
                body,
            });
        }

        info!("Successfully updated {} {}", sobject, id);
        Ok(())
    }

    /// Delete a record
    pub async fn delete(&self, sobject: &str, id: &str) -> SfResult<()> {
        let url = format!(
            "{}/services/data/v57.0/sobjects/{}/{}",
            self.base_url, sobject, id
        );

        debug!("Deleting {} record {}", sobject, id);

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(SfError::NotFound {
                sobject: sobject.to_string(),
                id: id.to_string(),
            });
        }

        if !status.is_success() {
            let body = response.text().await?;
            return Err(SfError::Api {
                status: status.as_u16(),
                body,
            });
        }

        info!("Successfully deleted {} {}", sobject, id);
        Ok(())
    }

    /// Upsert a record (insert or update based on external ID)
    ///
    /// # Example
    /// ```ignore
    /// let upsert = UpsertBuilder::new("ExternalId__c", "ext-12345");
    /// client.upsert("Account", upsert, &account_data).await?;
    /// ```
    pub async fn upsert<T: Serialize>(
        &self,
        sobject: &str,
        builder: UpsertBuilder,
        data: &T,
    ) -> SfResult<InsertResponse> {
        let url = format!(
            "{}/services/data/v57.0/sobjects/{}/{}/{}",
            self.base_url, sobject, builder.external_id_field, builder.external_id_value
        );

        debug!(
            "Upserting {} record with external ID {}",
            sobject, builder.external_id_value
        );

        let response = self
            .http_client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            return Err(SfError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let upsert_response: InsertResponse = response.json().await?;
        info!(
            "Successfully upserted {} with ID: {}",
            sobject, upsert_response.id
        );

        Ok(upsert_response)
    }
}
