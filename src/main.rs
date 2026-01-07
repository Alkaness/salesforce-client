//! Example usage of the Salesforce API client
//!
//! This demonstrates how to use the type-driven client to query Salesforce
//! data with compile-time type safety.

use salesforce_client::{ClientConfig, SalesforceClient, SfError};
use serde::{Deserialize, Serialize};

/// Domain model for a Salesforce Account object.
///
/// **Key Design Patterns:**
/// 1. **Field renaming**: Salesforce uses PascalCase (Id, Name), but Rust
///    convention is snake_case. We use `#[serde(rename)]` to bridge this gap.
/// 2. **Optional fields**: Fields that might not always be present should be
///    wrapped in `Option<T>` to avoid deserialization errors.
/// 3. **Type safety**: Using proper Rust types (f64 for revenue) instead of
///    raw JSON values provides compile-time guarantees.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Account {
    /// Salesforce unique identifier (18-character ID)
    #[serde(rename = "Id")]
    id: String,

    /// Account name
    #[serde(rename = "Name")]
    name: String,

    /// Annual revenue in the org's default currency
    ///
    /// Wrapped in `Option` because:
    /// 1. Field might not be queried (SELECT doesn't include it)
    /// 2. Value might be NULL in Salesforce
    /// 3. User might not have field-level security access
    #[serde(rename = "AnnualRevenue")]
    annual_revenue: Option<f64>,
}

/// Example of a Contact object with relationship fields
///
/// Demonstrates how to handle Salesforce relationships in the type system.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
struct Contact {
    #[serde(rename = "Id")]
    id: String,

    #[serde(rename = "FirstName")]
    first_name: Option<String>,

    #[serde(rename = "LastName")]
    last_name: String,

    #[serde(rename = "Email")]
    email: Option<String>,

    /// Salesforce relationship field (foreign key to Account)
    #[serde(rename = "AccountId")]
    account_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), SfError> {
    // In production, these would come from environment variables or a config file
    // Example using std::env:
    //   let base_url = std::env::var("SF_INSTANCE_URL").expect("SF_INSTANCE_URL not set");
    //   let token = std::env::var("SF_ACCESS_TOKEN").expect("SF_ACCESS_TOKEN not set");

    let base_url = "https://yourinstance.salesforce.com";
    let access_token = "your_access_token_here";

    // Create the client with enterprise features enabled
    let config = ClientConfig::new(base_url, access_token);
    let client = SalesforceClient::new(config);

    println!("🔍 Querying Accounts from Salesforce...\n");

    // Example 1: Query accounts with all fields
    // The type annotation `Vec<Account>` tells the compiler and the `query` method
    // what type to deserialize into. This is type-driven development in action!
    let accounts: Vec<Account> = client
        .query("SELECT Id, Name, AnnualRevenue FROM Account LIMIT 5")
        .await?;

    println!("📊 Found {} accounts:", accounts.len());
    for account in &accounts {
        println!("  • {} (ID: {})", account.name, account.id);
        if let Some(revenue) = account.annual_revenue {
            println!("    Annual Revenue: ${:.2}", revenue);
        }
    }

    println!("\n---\n");

    // Example 2: Query contacts - demonstrates reusing the same client
    // for different types
    println!("🔍 Querying Contacts from Salesforce...\n");

    let contacts: Vec<Contact> = client
        .query("SELECT Id, FirstName, LastName, Email FROM Contact LIMIT 5")
        .await?;

    println!("👥 Found {} contacts:", contacts.len());
    for contact in &contacts {
        let full_name = match &contact.first_name {
            Some(first) => format!("{} {}", first, contact.last_name),
            None => contact.last_name.clone(),
        };
        println!("  • {} (ID: {})", full_name, contact.id);
        if let Some(email) = &contact.email {
            println!("    Email: {}", email);
        }
    }

    println!("\n✅ Query completed successfully!");

    Ok(())
}

// Additional examples demonstrating error handling patterns

/// Example showing how to handle specific error types
#[allow(dead_code)]
async fn example_error_handling(client: &SalesforceClient) {
    match client
        .query::<Account>("SELECT Id FROM Account LIMIT 1")
        .await
    {
        Ok(accounts) => {
            println!("Retrieved {} accounts", accounts.len());
        }
        Err(SfError::Network(e)) => {
            eprintln!("Network error: {}", e);
            // Could implement retry logic here
        }
        Err(SfError::Serialization(e)) => {
            eprintln!("Failed to deserialize response: {}", e);
            // Likely a mismatch between struct fields and SOQL query
        }
        Err(SfError::Api { status, body }) => {
            eprintln!("Salesforce API error ({}): {}", status, body);
            // Could parse the error body for specific Salesforce error codes
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
}

/// Example showing how to work with the client in a concurrent context
///
/// This demonstrates the idiomatic way to run multiple queries concurrently
/// in Rust using `tokio::join!` macro.
#[allow(dead_code)]
async fn example_concurrent_queries(client: SalesforceClient) -> Result<(), SfError> {
    // Clone is cheap because reqwest::Client uses Arc internally
    let client1 = client.clone();
    let client2 = client.clone();

    // Use tokio::join! for concurrent execution without spawning tasks
    // This is more efficient and avoids the nested Result from spawn
    let (accounts_result, contacts_result) = tokio::join!(
        client1.query::<Account>("SELECT Id, Name FROM Account LIMIT 10"),
        client2.query::<Contact>("SELECT Id, LastName FROM Contact LIMIT 10")
    );

    // Handle Results directly - no need to unwrap JoinError
    let accounts = accounts_result?;
    let contacts = contacts_result?;

    println!(
        "Fetched {} accounts and {} contacts concurrently",
        accounts.len(),
        contacts.len()
    );

    Ok(())
}
