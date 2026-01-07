//! Benchmarks for query performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use salesforce_client::{ClientConfig, SalesforceClient};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct Account {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Name")]
    name: String,
}

fn benchmark_query_parsing(c: &mut Criterion) {
    c.bench_function("query string parsing", |b| {
        b.iter(|| black_box("SELECT Id, Name FROM Account WHERE AnnualRevenue > 1000000"))
    });
}

fn benchmark_client_creation(c: &mut Criterion) {
    c.bench_function("client creation", |b| {
        b.iter(|| {
            let config = ClientConfig::new("https://test.salesforce.com", "test_token");
            black_box(SalesforceClient::new(config))
        })
    });
}

criterion_group!(benches, benchmark_query_parsing, benchmark_client_creation);
criterion_main!(benches);
