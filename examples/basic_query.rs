//! Basic query example - demonstrates the simplest usage pattern

use salesforce_client::{SalesforceClient, SfError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Account {
    #[serde(rename = "Id")]
    id: String,

    #[serde(rename = "Name")]
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), SfError> {
    // Get credentials from environment
    let base_url = std::env::var("SF_INSTANCE_URL")
        .unwrap_or_else(|_| "https://yourinstance.salesforce.com".to_string());
    let token = std::env::var("SF_ACCESS_TOKEN").unwrap_or_else(|_| "your_token_here".to_string());

    let client = SalesforceClient::new(base_url, token);

    // Simple type-driven query
    let accounts: Vec<Account> = client
        .query("SELECT Id, Name FROM Account LIMIT 10")
        .await?;

    println!("Found {} accounts:", accounts.len());
    for account in accounts {
        println!("  • {} ({})", account.name, account.id);
    }

    Ok(())
}
