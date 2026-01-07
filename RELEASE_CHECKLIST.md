# GitHub Release Checklist

## Pre-Release Verification

### Build and Tests
- [x] `cargo build --release` completes successfully
- [x] `cargo test --lib` passes (with acceptable warnings)
- [x] `cargo clippy --all-targets --all-features` runs clean
- [x] `cargo fmt --all` applied
- [x] Documentation builds: `cargo doc --no-deps`

### Project Files
- [x] LICENSE-MIT created
- [x] LICENSE-APACHE created
- [x] README.md comprehensive and professional
- [x] CONTRIBUTING.md with guidelines
- [x] CHANGELOG.md with version history
- [x] .gitignore configured
- [x] Cargo.toml metadata complete

### GitHub Configuration
- [x] .github/workflows/ci.yml (CI pipeline)
- [x] .github/ISSUE_TEMPLATE/bug_report.md
- [x] .github/ISSUE_TEMPLATE/feature_request.md
- [x] .github/PULL_REQUEST_TEMPLATE.md
- [x] .github/FUNDING.yml (optional, edit as needed)

### Code Quality
- [x] No unsafe blocks
- [x] Comprehensive error handling
- [x] Structured logging via tracing
- [x] Examples compile and work
- [x] Benchmarks configured

## Repository Setup Steps

### 1. Create GitHub Repository

```bash
# On GitHub.com:
# 1. Click "New repository"
# 2. Name: salesforce-client
# 3. Description: "Production-grade Salesforce REST API client for Rust"
# 4. Public repository
# 5. DO NOT initialize with README (we have one)
# 6. Create repository
```

### 2. Initialize Git and Push

```bash
# Initialize git repository
git init

# Add all files
git add .

# Create initial commit
git commit -m "feat: initial release v0.2.0

- OAuth 2.0 with automatic token refresh
- Intelligent caching with TTL/TTI
- Retry logic with exponential backoff
- Rate limiting
- Automatic pagination
- Type-safe query builder
- Full CRUD operations
- Comprehensive documentation
- Zero unsafe code
- 9 production modules"

# Add remote (replace with your username)
git remote add origin https://github.com/yourusername/salesforce-client.git

# Push to GitHub
git branch -M main
git push -u origin main
```

### 3. Configure Repository Settings

On GitHub repository settings:

#### General
- [ ] Add topics: `rust`, `salesforce`, `api-client`, `oauth`, `async`, `tokio`
- [ ] Add description from Cargo.toml
- [ ] Enable Issues
- [ ] Enable Discussions (optional)

#### Branches
- [ ] Set `main` as default branch
- [ ] Add branch protection rules:
  - Require pull request reviews
  - Require status checks (CI)
  - Require linear history

#### Actions
- [ ] Enable GitHub Actions
- [ ] Verify CI workflow runs on push

### 4. Create Release Tag

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0

Enterprise-grade Salesforce API client with:
- OAuth auto-refresh
- Intelligent caching
- Retry logic
- Rate limiting
- Auto-pagination
- Type-safe query builder
- Full CRUD operations"

# Push tag
git push origin v0.2.0
```

### 5. Create GitHub Release

On GitHub:
1. Go to "Releases" → "Create a new release"
2. Choose tag: v0.2.0
3. Release title: "v0.2.0 - Enterprise Edition"
4. Description: Copy from CHANGELOG.md
5. Publish release

### 6. Publish to crates.io (Optional)

```bash
# Login to crates.io
cargo login

# Dry run
cargo publish --dry-run

# Publish
cargo publish
```

## Post-Release Tasks

### Documentation
- [ ] Verify docs.rs builds correctly
- [ ] Update badges in README if published to crates.io
- [ ] Create getting started tutorial (optional)

### Community
- [ ] Announce on:
  - Reddit: r/rust
  - Rust Users Forum
  - Twitter/X
  - Rust Discord
- [ ] Submit to This Week in Rust (optional)

### Monitoring
- [ ] Watch for issues
- [ ] Respond to community feedback
- [ ] Plan next version features

## Quick Commands Reference

```bash
# Verify everything before push
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build --release
cargo doc --no-deps

# Check Cargo.toml
cargo package --list

# View what will be published
cargo package --allow-dirty

# Create and push tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# Publish to crates.io
cargo publish
```

## Repository URL Structure

Update these in Cargo.toml after creating repository:
- `repository = "https://github.com/YOUR_USERNAME/salesforce-client"`
- `homepage = "https://github.com/YOUR_USERNAME/salesforce-client"`
- `documentation = "https://docs.rs/salesforce-client"`

## README Badges to Add (After Publishing)

```markdown
[![Crates.io](https://img.shields.io/crates/v/salesforce-client.svg)](https://crates.io/crates/salesforce-client)
[![Documentation](https://docs.rs/salesforce-client/badge.svg)](https://docs.rs/salesforce-client)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Build Status](https://github.com/YOUR_USERNAME/salesforce-client/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/salesforce-client/actions)
```

## Files Ready for GitHub

```
.
├── .github/
│   ├── workflows/
│   │   └── ci.yml
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── FUNDING.yml
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── auth.rs
│   ├── cache.rs
│   ├── crud.rs
│   ├── error.rs
│   ├── pagination.rs
│   ├── query_builder.rs
│   ├── rate_limit.rs
│   └── retry.rs
├── examples/
│   ├── basic_query.rs
│   ├── concurrent_queries.rs
│   ├── error_handling.rs
│   ├── relationships.rs
│   └── type_driven.rs
├── benches/
│   └── query_benchmark.rs
├── .gitignore
├── Cargo.toml
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE-MIT
├── LICENSE-APACHE
└── README.md
```

## Success Criteria

- [ ] Repository is public and accessible
- [ ] CI pipeline runs and passes
- [ ] README displays correctly on GitHub
- [ ] All files are committed
- [ ] Release tag is created
- [ ] GitHub release is published
- [ ] Documentation is clear and professional
- [ ] Project is discoverable via search

## Current Status

**Ready for GitHub push!** All files are prepared and verified.

**Next step:** Execute "Repository Setup Steps" above.
