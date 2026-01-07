---
name: Bug Report
about: Report a bug or unexpected behavior
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description

A clear and concise description of the bug.

## To Reproduce

Steps to reproduce the behavior:
1. Configure client with '...'
2. Execute query '...'
3. See error

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened.

## Code Example

```rust
// Minimal reproducible example
let client = SalesforceClient::new(config);
let result = client.query("...").await?;
```

## Environment

- Rust version: [output of `rustc --version`]
- salesforce-client version: [e.g., 0.2.0]
- Operating System: [e.g., Ubuntu 22.04]
- Tokio version: [e.g., 1.41.0]

## Error Messages

```
Paste any error messages or stack traces here
```

## Additional Context

Any other context about the problem.
