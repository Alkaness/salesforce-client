//! Type-driven design example - demonstrates advanced Rust patterns
//!
//! This example shows how to use Rust's type system to create safe,
//! ergonomic abstractions over Salesforce data.

use salesforce_client::{ClientConfig, SalesforceClient, SfError};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

// ============================================================================
// Pattern 1: Newtype wrappers for type safety
// ============================================================================

/// Newtype wrapper for Salesforce IDs
///
/// **Why?** Prevents mixing up different types of IDs (Account vs Contact)
/// and provides a place to add validation logic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
#[allow(dead_code)]
struct SalesforceId(String);

impl SalesforceId {
    /// Validates that the ID is the correct length (15 or 18 characters)
    #[allow(dead_code)]
    fn is_valid(&self) -> bool {
        self.0.len() == 15 || self.0.len() == 18
    }
}

// ============================================================================
// Pattern 2: Phantom types for compile-time guarantees
// ============================================================================

/// Marker types for different Salesforce objects
#[derive(Debug)]
#[allow(dead_code)]
struct AccountMarker;
#[derive(Debug)]
#[allow(dead_code)]
struct ContactMarker;
#[derive(Debug)]
#[allow(dead_code)]
struct OpportunityMarker;

/// Type-safe ID that knows which object it refers to
///
/// **Benefit:** Can't accidentally pass an Account ID where a Contact ID is expected
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
struct TypedId<T> {
    id: String,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> TypedId<T> {
    fn new(id: String) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    fn as_str(&self) -> &str {
        &self.id
    }
}

// Type aliases for better ergonomics
type AccountId = TypedId<AccountMarker>;
type ContactId = TypedId<ContactMarker>;

// ============================================================================
// Pattern 3: Builder pattern for complex queries
// ============================================================================

/// Query builder for type-safe SOQL construction
struct AccountQueryBuilder {
    fields: Vec<&'static str>,
    limit: Option<u32>,
    order_by: Option<&'static str>,
}

impl AccountQueryBuilder {
    fn new() -> Self {
        Self {
            fields: vec!["Id"], // Id is always included
            limit: None,
            order_by: None,
        }
    }

    fn with_name(mut self) -> Self {
        self.fields.push("Name");
        self
    }

    fn with_revenue(mut self) -> Self {
        self.fields.push("AnnualRevenue");
        self
    }

    fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    fn order_by(mut self, field: &'static str) -> Self {
        self.order_by = Some(field);
        self
    }

    fn build(self) -> String {
        let mut query = format!("SELECT {} FROM Account", self.fields.join(", "));

        if let Some(order) = self.order_by {
            query.push_str(&format!(" ORDER BY {}", order));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        query
    }
}

// ============================================================================
// Pattern 4: From/Into traits for conversions
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AccountDto {
    #[serde(rename = "Id")]
    id: String,

    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "AnnualRevenue")]
    annual_revenue: Option<f64>,
}

/// Domain model (what your application uses)
#[derive(Debug)]
struct Account {
    id: AccountId,
    name: String,
    annual_revenue: Option<f64>,
}

/// Implement From trait for clean conversion from DTO to domain model
impl From<AccountDto> for Account {
    fn from(dto: AccountDto) -> Self {
        Self {
            id: TypedId::new(dto.id),
            name: dto.name,
            annual_revenue: dto.annual_revenue,
        }
    }
}

// ============================================================================
// ============================================================================

/// Helper trait for working with query results

#[allow(dead_code)]
trait QueryResultExt<T> {
    /// Filter and transform in a single iterator chain
    fn into_domain_models<U>(self) -> Vec<U>
    where
        U: From<T>;
}

impl<T> QueryResultExt<T> for Vec<T> {
    fn into_domain_models<U>(self) -> Vec<U>
    where
        U: From<T>,
    {
        self.into_iter().map(U::from).collect()
    }
}

// ============================================================================
// Main example showing all patterns in action
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), SfError> {
    let base_url = std::env::var("SF_INSTANCE_URL")
        .unwrap_or_else(|_| "https://yourinstance.salesforce.com".to_string());
    let token = std::env::var("SF_ACCESS_TOKEN").unwrap_or_else(|_| "your_token_here".to_string());

    let config = ClientConfig::new(base_url, token);
    let client = SalesforceClient::new(config);

    // Pattern: Builder for type-safe query construction
    let query = AccountQueryBuilder::new()
        .with_name()
        .with_revenue()
        .order_by("Name")
        .limit(10)
        .build();

    println!("📝 Generated SOQL: {}\n", query);

    // Query returns DTOs
    let account_dtos: Vec<AccountDto> = client.query(&query).await?;

    // Pattern: Iterator chain with From trait for transformation
    let accounts: Vec<Account> = account_dtos.into_iter().map(Account::from).collect();

    // Or using our extension trait:
    // let accounts: Vec<Account> = account_dtos.into_domain_models();

    println!("✅ Retrieved {} accounts:\n", accounts.len());

    // Pattern: Iterator chains for filtering and processing
    let high_revenue_accounts = accounts
        .iter()
        .filter(|a| a.annual_revenue.unwrap_or(0.0) > 1_000_000.0)
        .collect::<Vec<_>>();

    println!(
        "💰 High-revenue accounts (>$1M): {}",
        high_revenue_accounts.len()
    );

    for account in high_revenue_accounts {
        println!("  • {} (ID: {})", account.name, account.id.as_str());
        if let Some(revenue) = account.annual_revenue {
            println!("    Revenue: ${:.2}", revenue);
        }
    }

    // Pattern: Type-safe IDs prevent mixing different object types
    let _account_id: AccountId = AccountId::new("001xx000003DGbXXXX".to_string());
    let _contact_id: ContactId = ContactId::new("003xx000004TmiAAAS".to_string());

    // This would be a compile error:
    // process_account(_contact_id); // ❌ Type mismatch!

    Ok(())
}

/// Example function that only accepts Account IDs
#[allow(dead_code)]
fn process_account(_id: TypedId<AccountMarker>) {
    // Implementation here
}
