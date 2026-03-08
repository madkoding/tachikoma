# Testing Guide

This document explains how to run tests in the TACHIKOMA-OS project.

## Rust Tests

### Run All Tests

```bash
# Run tests for all services
cargo test --workspace
```

### Run Tests for Specific Service

```bash
# Navigate to service directory
cd tachikoma-backend

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_memory_node_creation

# Run tests matching pattern
cargo test test_memory

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Run Integration Tests

```bash
cd tachikoma-backend

# Run only integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test integration_tests test_health_endpoint
```

### Run Tests with Lint

```bash
# Run clippy before tests
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## TypeScript/React Tests

### Setup

First, install dependencies:

```bash
cd tachikoma-ui
npm install
```

### Run Tests

```bash
# Run tests in watch mode
npm run test

# Run tests once
npm run test:run

# Run tests with coverage
npm run test:coverage

# Run specific test file
npm run test -- ChatInput.test.tsx

# Run tests matching pattern
npm run test -- --reporter=verbose
```

### Test Structure

Tests are located alongside the components they test:

```
tachikoma-ui/src/
├── components/
│   ├── common/
│   │   ├── Modal.tsx
│   │   └── Modal.test.tsx
│   ├── ChatInput.tsx
│   └── ChatInput.test.tsx
├── tests/
│   └── setup.ts
└── ...
```

## CI/CD

Tests run automatically on:
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`

Check the [CI workflow](.github/workflows/ci.yml) for details.

## Writing Tests

### Rust Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let result = function_under_test();
        assert_eq!(result, expected_value);
    }
}
```

### Rust Integration Tests

```rust
use tachikoma_backend::*;

#[tokio::test]
async fn test_api_endpoint() {
    let app = create_test_app().await;
    let (status, body) = make_request(&mut app, "GET", "/api/health", None).await;
    
    assert_eq!(status, StatusCode::OK);
}
```

### React Component Tests

```tsx
import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'

describe('ComponentName', () => {
  it('should do something', () => {
    render(<ComponentName />)
    expect(screen.getByText('text')).toBeInTheDocument()
  })
})
```

## Coverage Reports

### Rust

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cd tachikoma-backend
cargo tarpaulin --out Html

# Open report
open ./tarpaulin-report.html
```

### TypeScript

```bash
cd tachikoma-ui
npm run test:coverage

# Open HTML report
open ./coverage/index.html
```

## Troubleshooting

### Rust Tests Fail

1. Ensure all dependencies are built: `cargo build`
2. Check for compilation errors: `cargo check`
3. Run with verbose output: `cargo test --verbose`

### TypeScript Tests Fail

1. Clear node_modules: `rm -rf node_modules && npm install`
2. Check TypeScript compilation: `npm run type-check`
3. Run with debug output: `npm run test -- --reporter=verbose`

### Integration Tests Fail

Integration tests may require:
- SurrealDB running (or use in-memory mode)
- Ollama running (for LLM tests)
- Network access (for HTTP client tests)

Run with isolated unit tests instead:
```bash
cargo test --lib
```
