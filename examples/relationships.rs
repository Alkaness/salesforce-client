//! Relationships example - demonstrates querying Salesforce relationships

use salesforce_client::{SalesforceClient, SfError};
use serde::Deserialize;

/// Contact with parent Account relationship
#[derive(Debug, Deserialize)]
struct Contact {
    #[serde(rename = "Id")]
    id: String,

    #[serde(rename = "FirstName")]
    first_name: Option<String>,

    #[serde(rename = "LastName")]
    last_name: String,

    #[serde(rename = "Email")]
    email: Option<String>,

    // Foreign key to Account
    #[serde(rename = "AccountId")]
    account_id: Option<String>,

    // Parent relationship (query with Account.Name)
    #[serde(rename = "Account")]
    account: Option<AccountReference>,
}

/// Nested Account data accessed through relationship
#[derive(Debug, Deserialize)]
struct AccountReference {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "Industry")]
    industry: Option<String>,
}

/// Account with child Contacts relationship
#[derive(Debug, Deserialize)]
struct AccountWithContacts {
    #[serde(rename = "Id")]
    id: String,

    #[serde(rename = "Name")]
    name: String,

    // Child relationship (subquery)
    #[serde(rename = "Contacts")]
    contacts: Option<ContactsSubquery>,
}

/// Wrapper for child relationship queries
#[derive(Debug, Deserialize)]
struct ContactsSubquery {
    records: Vec<ContactSummary>,
}

#[derive(Debug, Deserialize)]
struct ContactSummary {
    #[serde(rename = "Id")]
    id: String,

    #[serde(rename = "Email")]
    email: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), SfError> {
    let base_url = std::env::var("SF_INSTANCE_URL")
        .unwrap_or_else(|_| "https://yourinstance.salesforce.com".to_string());
    let token = std::env::var("SF_ACCESS_TOKEN").unwrap_or_else(|_| "your_token_here".to_string());

    let client = SalesforceClient::new(base_url, token);

    println!("📊 Querying parent relationships (Contact → Account)...\n");

    // Parent relationship query
    let contacts: Vec<Contact> = client
        .query(
            "SELECT Id, FirstName, LastName, Email, AccountId, \
             Account.Name, Account.Industry \
             FROM Contact \
             WHERE AccountId != NULL \
             LIMIT 5",
        )
        .await?;

    for contact in contacts {
        let full_name = match &contact.first_name {
            Some(first) => format!("{} {}", first, contact.last_name),
            None => contact.last_name.clone(),
        };

        println!("👤 {}", full_name);
        if let Some(email) = &contact.email {
            println!("   Email: {}", email);
        }
        if let Some(account) = &contact.account {
            println!("   Account: {}", account.name);
            if let Some(industry) = &account.industry {
                println!("   Industry: {}", industry);
            }
        }
        println!();
    }

    println!("---\n");
    println!("📊 Querying child relationships (Account → Contacts)...\n");

    // Child relationship query (subquery)
    let accounts: Vec<AccountWithContacts> = client
        .query(
            "SELECT Id, Name, \
             (SELECT Id, Email FROM Contacts LIMIT 3) \
             FROM Account \
             WHERE Id IN (SELECT AccountId FROM Contact WHERE Email != NULL) \
             LIMIT 3",
        )
        .await?;

    for account in accounts {
        println!("🏢 {}", account.name);

        if let Some(contacts_subquery) = &account.contacts {
            if !contacts_subquery.records.is_empty() {
                println!("   Contacts:");
                for contact in &contacts_subquery.records {
                    if let Some(email) = &contact.email {
                        println!("     • {} ({})", email, contact.id);
                    }
                }
            }
        }
        println!();
    }

    Ok(())
}
