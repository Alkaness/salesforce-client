//! Concurrent queries example - demonstrates parallel query execution

use salesforce_client::{ClientConfig, SalesforceClient, SfError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Account {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Contact {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Email")]
    email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Opportunity {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "StageName")]
    stage_name: String,
}

#[tokio::main]
async fn main() -> Result<(), SfError> {
    let base_url = std::env::var("SF_INSTANCE_URL")
        .unwrap_or_else(|_| "https://yourinstance.salesforce.com".to_string());
    let token = std::env::var("SF_ACCESS_TOKEN").unwrap_or_else(|_| "your_token_here".to_string());

    let config = ClientConfig::new(base_url, token);
    let client = SalesforceClient::new(config);

    println!("🚀 Executing 3 queries concurrently...\n");

    let start = std::time::Instant::now();

    // Clone is cheap - reqwest::Client uses Arc internally
    let client1 = client.clone();
    let client2 = client.clone();
    let client3 = client.clone();

    // Execute all queries concurrently using tokio::join!
    // This is the idiomatic Rust way for concurrent operations
    let (accounts_result, contacts_result, opportunities_result) = tokio::join!(
        client1.query::<Account>("SELECT Id, Name FROM Account LIMIT 100"),
        client2.query::<Contact>("SELECT Id, Email FROM Contact LIMIT 100"),
        client3.query::<Opportunity>("SELECT Id, StageName FROM Opportunity LIMIT 100")
    );

    let elapsed = start.elapsed();

    // Handle results
    let accounts = accounts_result?;
    let contacts = contacts_result?;
    let opportunities = opportunities_result?;

    println!("✅ All queries completed in {:?}\n", elapsed);
    println!("Results:");
    println!("  • Accounts: {}", accounts.len());
    println!("  • Contacts: {}", contacts.len());
    println!("  • Opportunities: {}", opportunities.len());

    Ok(())
}
