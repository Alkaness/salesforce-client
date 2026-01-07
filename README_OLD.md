# Salesforce API Client for Rust

A lightweight, type-driven Salesforce REST API client library built with idiomatic Rust patterns.

## 🎯 Philosophy

This library solves the lack of ergonomic "batteries-included" Salesforce connectors in the Rust ecosystem by providing:

- **Type Safety**: Generic query methods that deserialize directly into your domain types
- **Zero-cost Abstractions**: Leverages Rust's type system for compile-time guarantees
- **Async First**: Built on `tokio` and `reqwest` for high-performance async I/O
- **Idiomatic Error Handling**: Uses `Result` and `thiserror` - no `.unwrap()` or `.expect()` in production code

## 🚀 Quick Start

### Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
salesforce-client = { path = "." }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Basic Usage

```rust
use salesforce_client::{SalesforceClient, SfError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Account {
    #[serde(rename = "Id")]
    id: String,
    
    #[serde(rename = "Name")]
    name: String,
    
    #[serde(rename = "AnnualRevenue")]
    annual_revenue: Option<f64>,
}

#[tokio::main]
async fn main() -> Result<(), SfError> {
    let client = SalesforceClient::new(
        "https://yourinstance.salesforce.com",
        "your_access_token",
    );

    // Type-driven query: compiler knows to deserialize into Vec<Account>
    let accounts: Vec<Account> = client
        .query("SELECT Id, Name, AnnualRevenue FROM Account LIMIT 10")
        .await?;

    for account in accounts {
        println!("{}: ${:?}", account.name, account.annual_revenue);
    }

    Ok(())
}
```

## 🏗️ Architecture

### Core Components

#### `SalesforceClient`

The main client struct that holds:
- `base_url`: Your Salesforce instance URL
- `access_token`: OAuth 2.0 bearer token
- `http_client`: Reusable `reqwest::Client` with connection pooling

```rust
pub struct SalesforceClient {
    base_url: String,
    access_token: String,
    http_client: reqwest::Client,
}
```

**Design Decision**: The client is `Clone`-able because `reqwest::Client` uses `Arc` internally, making it cheap to share across async tasks.

#### Generic `query` Method

```rust
pub async fn query<T>(&self, soql: impl AsRef<str>) -> Result<Vec<T>, SfError>
where
    T: DeserializeOwned,
```

**Key Features**:
- Generic over return type `T` (must implement `serde::Deserialize`)
- Accepts any SOQL query string
- Handles Salesforce's response wrapper internally
- Returns clean `Vec<T>` to the user

#### Error Handling with `SfError`

```rust
#[derive(Debug, Error)]
pub enum SfError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("API error (status {status}): {body}")]
    Api { status: u16, body: String },
}
```

**Why `thiserror`?**: Provides automatic `Display`, `Error`, and `From` implementations, making error propagation with `?` seamless.

## 📚 Advanced Examples

### Handling Salesforce Relationships

```rust
#[derive(Debug, Deserialize)]
struct Contact {
    #[serde(rename = "Id")]
    id: String,
    
    #[serde(rename = "AccountId")]
    account_id: Option<String>,
    
    // Querying related objects (parent relationship)
    #[serde(rename = "Account")]
    account: Option<AccountRef>,
}

#[derive(Debug, Deserialize)]
struct AccountRef {
    #[serde(rename = "Name")]
    name: String,
}

// Query with relationship
let contacts: Vec<Contact> = client
    .query("SELECT Id, AccountId, Account.Name FROM Contact LIMIT 5")
    .await?;
```

### Concurrent Queries

```rust
let client1 = client.clone(); // Cheap clone
let client2 = client.clone();

let (accounts, contacts) = tokio::join!(
    client1.query::<Account>("SELECT Id, Name FROM Account LIMIT 100"),
    client2.query::<Contact>("SELECT Id, Email FROM Contact LIMIT 100")
);

let accounts = accounts?;
let contacts = contacts?;
```

### Error Handling Patterns

```rust
match client.query::<Account>("SELECT Id FROM Account").await {
    Ok(accounts) => println!("Found {} accounts", accounts.len()),
    
    Err(SfError::Network(e)) => {
        eprintln!("Connection failed: {}", e);
        // Implement retry logic
    }
    
    Err(SfError::Serialization(e)) => {
        eprintln!("Type mismatch: {}", e);
        // Check if struct fields match SOQL query
    }
    
    Err(SfError::Api { status, body }) => {
        eprintln!("Salesforce error {}: {}", status, body);
        // Handle specific API errors (invalid query, auth failure, etc.)
    }
}
```

## 🔒 Security Considerations

### Token Management

The current implementation stores the access token as a `String`. For production use, consider:

1. **Environment Variables**:
   ```rust
   let token = std::env::var("SF_ACCESS_TOKEN")
       .expect("SF_ACCESS_TOKEN must be set");
   ```

2. **Secret Management Services**: Use AWS Secrets Manager, HashiCorp Vault, etc.

3. **Token Refresh**: Implement OAuth refresh token flow (future enhancement)

### TLS Configuration

The library uses `rustls-tls` instead of native TLS for:
- **Memory safety**: Pure Rust implementation
- **Cross-platform**: No OpenSSL dependency issues
- **Smaller attack surface**: Reduced C FFI

## 🧪 Testing

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run

# Build release version
cargo build --release
```

## 🔧 Extending the Library

### Adding New Methods

To add support for other Salesforce APIs (insert, update, delete):

```rust
impl SalesforceClient {
    pub async fn insert<T>(&self, sobject: &str, data: &T) -> Result<String, SfError>
    where
        T: serde::Serialize,
    {
        let url = format!("{}/services/data/v57.0/sobjects/{}", self.base_url, sobject);
        
        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(data)
            .send()
            .await?;
            
        // ... handle response and extract ID
        todo!()
    }
}
```

### Custom Deserialization

For complex Salesforce types, implement custom `Deserialize`:

```rust
#[derive(Debug)]
struct SalesforceDate(chrono::NaiveDate);

impl<'de> Deserialize<'de> for SalesforceDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let date = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(serde::de::Error::custom)?;
        Ok(SalesforceDate(date))
    }
}
```

## 📊 Performance Characteristics

- **Connection Pooling**: `reqwest::Client` reuses connections
- **Zero-copy Deserialization**: `serde_json` uses efficient parsing
- **Async I/O**: Non-blocking requests via `tokio`
- **Memory**: Stack-allocated where possible, heap only for dynamic data

## 🛠️ Dependencies

| Crate | Purpose | Why This Choice? |
|-------|---------|------------------|
| `tokio` | Async runtime | Industry standard, mature, excellent ecosystem |
| `reqwest` | HTTP client | Built on `hyper`, supports async, connection pooling |
| `serde` | Serialization | De facto standard for Rust serialization |
| `thiserror` | Error types | Ergonomic derive macros for library errors |

## 📝 License

MIT OR Apache-2.0

## 🤝 Contributing

Contributions welcome! Focus areas:
- OAuth token refresh flow
- Bulk API support
- Streaming API integration
- Composite API operations
- Query pagination for large result sets

---

**Built with ❤️ using idiomatic Rust patterns**
