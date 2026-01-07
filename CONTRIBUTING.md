# Contributing to Salesforce API Client

Thank you for your interest in contributing to this project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/salesforce-client`
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Run formatting: `cargo fmt`
7. Run linter: `cargo clippy -- -D warnings`
8. Commit your changes: `git commit -m "Add your feature"`
9. Push to your fork: `git push origin feature/your-feature-name`
10. Open a Pull Request

## Code Standards

### Rust Best Practices

- Follow the Rust API Guidelines
- Use `cargo fmt` with default settings
- Address all `cargo clippy` warnings
- Write idiomatic Rust code
- Avoid `unsafe` code unless absolutely necessary (document with `# Safety`)

### Error Handling

- Never use `.unwrap()` or `.expect()` in library code
- Use `Result` types for fallible operations
- Provide context-rich error messages
- Use the `?` operator for error propagation

### Documentation

- Document all public APIs with `///` comments
- Include examples in documentation
- Explain complex algorithms or design decisions
- Update README.md if adding features

### Testing

- Write unit tests for new functionality
- Add integration tests for API interactions
- Ensure all tests pass before submitting PR
- Maintain or improve code coverage

### Commit Messages

Follow conventional commits:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `refactor:` Code refactoring
- `test:` Test additions or modifications
- `chore:` Maintenance tasks

Example: `feat: add bulk API v2 support`

## Pull Request Process

1. Ensure your code builds without errors or warnings
2. Update documentation for any API changes
3. Add tests for new functionality
4. Update CHANGELOG.md with your changes
5. Request review from maintainers
6. Address review feedback
7. Wait for approval and merge

## Areas for Contribution

### High Priority

- Bulk API v2.0 implementation
- Streaming API support
- Additional query builder features
- Performance optimizations
- Integration tests

### Medium Priority

- Composite API operations
- Additional caching strategies
- GraphQL support
- CLI tool
- More examples

### Documentation

- Tutorials and guides
- Architecture diagrams
- Performance benchmarking
- Migration guides
- API reference improvements

## Code Review

All submissions require review. We use GitHub pull requests for this purpose.

Reviewers will check for:
- Code quality and style
- Test coverage
- Documentation completeness
- API design consistency
- Performance implications

## Reporting Issues

When reporting bugs, include:
- Rust version (`rustc --version`)
- Library version
- Minimal reproducible example
- Expected vs actual behavior
- Error messages and stack traces

## Questions

For questions or discussions:
- Open a GitHub issue with the "question" label
- Provide context and specific examples
- Be respectful and constructive

## License

By contributing, you agree that your contributions will be licensed under both MIT and Apache-2.0 licenses.

## Code of Conduct

Be professional, respectful, and constructive in all interactions.
