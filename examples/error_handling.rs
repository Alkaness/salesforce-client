//! Error handling example - demonstrates proper error handling patterns

use salesforce_client::{SalesforceClient, SfError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Account {
    #[serde(rename = "Id")]
    id: String,
}

#[tokio::main]
async fn main() {
    let client = SalesforceClient::new("https://test.salesforce.com", "fake_token_for_testing");

    // Demonstrate exhaustive error matching
    match client
        .query::<Account>("SELECT Id FROM Account LIMIT 1")
        .await
    {
        Ok(accounts) => {
            println!("✅ Success! Retrieved {} accounts", accounts.len());
        }

        Err(SfError::Network(e)) => {
            eprintln!("❌ Network error: {}", e);
            eprintln!("   This could be due to:");
            eprintln!("   - Connection failure");
            eprintln!("   - DNS resolution issues");
            eprintln!("   - Timeout");
            eprintln!("\n   Consider implementing retry logic with exponential backoff");
        }

        Err(SfError::Serialization(e)) => {
            eprintln!("❌ Serialization error: {}", e);
            eprintln!("   This usually means:");
            eprintln!("   - Struct fields don't match the SOQL query");
            eprintln!("   - Field types are incorrect (e.g., String vs Number)");
            eprintln!("   - Missing #[serde(rename = \"...\")]");
        }

        Err(SfError::Api { status, body }) => {
            eprintln!("❌ Salesforce API error (status {}):", status);
            eprintln!("   Response: {}", body);

            match status {
                401 => eprintln!("   → Authentication failed. Check your access token."),
                403 => {
                    eprintln!("   → Forbidden. Check field-level security and object permissions.")
                }
                404 => eprintln!("   → Object or endpoint not found."),
                400 => eprintln!("   → Bad request. Check your SOQL syntax."),
                _ => eprintln!("   → Unexpected API error."),
            }
        }
    }
}
