# Contributing to rccusage

Thank you for your interest in contributing to rccusage! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions.

## How to Contribute

### Reporting Issues

- Check if the issue already exists
- Provide a clear description of the problem
- Include steps to reproduce the issue
- Mention your OS, Rust version, and rccusage version

### Submitting Pull Requests

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/yourusername/rccusage.git
   cd rccusage
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Follow the existing code style
   - Add tests for new functionality
   - Update documentation as needed

4. **Run tests and checks**
   ```bash
   # Run tests
   cargo test

   # Check formatting
   cargo fmt -- --check

   # Run clippy
   cargo clippy -- -D warnings

   # Build in release mode
   cargo build --release
   ```

5. **Commit your changes**
   ```bash
   git commit -m "feat: add new feature"
   ```
   Follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `test:` for test additions/changes
   - `refactor:` for code refactoring
   - `perf:` for performance improvements
   - `chore:` for maintenance tasks

6. **Push and create a pull request**
   ```bash
   git push origin feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Cargo
- Git

### Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run with debug output
LOG_LEVEL=3 cargo run -- daily
```

### Running Tests

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

### Code Style

- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Follow Rust naming conventions:
  - `snake_case` for functions and variables
  - `PascalCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

### Project Structure

```
src/
├── main.rs           # Entry point
├── commands/         # CLI command implementations
├── types/            # Type definitions
├── aggregation.rs    # Data aggregation logic
├── data_loader.rs    # JSONL file loading
├── pricing.rs        # Cost calculation
├── output/           # Table and JSON formatting
└── utils.rs          # Utility functions
```

## Testing

### Unit Tests

Place unit tests in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        // Test code here
    }
}
```

### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/integration_test.rs
use rccusage::*;

#[test]
fn test_integration() {
    // Test code here
}
```

## Performance Considerations

- Use streaming for large files (already implemented)
- Minimize allocations in hot paths
- Use parallel processing where appropriate (Rayon)
- Profile before optimizing

## Documentation

- Add doc comments for public APIs:
  ```rust
  /// Calculates the cost for the given tokens.
  ///
  /// # Arguments
  /// * `tokens` - The token count
  ///
  /// # Returns
  /// The calculated cost as a Decimal
  pub fn calculate_cost(tokens: u64) -> Decimal {
      // Implementation
  }
  ```

- Update README.md for user-facing changes
- Update CHANGELOG.md for all changes

## Release Process

Releases are managed by maintainers:

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create and push a tag: `git tag v0.1.0`
4. GitHub Actions will build and release binaries

## Getting Help

- Open an issue for questions
- Check existing issues and PRs
- Review the documentation

## License

By contributing, you agree that your contributions will be licensed under the MIT License.