# Test Configuration for PugVault 🐶

## Test Suites

### Unit Tests (`cargo test --lib`)
- **Location**: `src/lib.rs` 
- **Coverage**: Core vault functionality, encryption/decryption, key derivation
- **Tests**: 13 tests covering crypto operations, data serialization, error handling
- **Purpose**: Fast feedback loop for core business logic

### Integration Tests (`cargo test --test integration_tests`)
- **Location**: `tests/integration_tests.rs`
- **Coverage**: End-to-end CLI functionality  
- **Tests**: 8 tests covering CLI commands (set, get, list, delete, change-password)
- **Purpose**: Verify user-facing functionality works correctly
- **Requirements**: Needs `PUG_MASTER_PASSWORD` environment variable

### Security Tests (`cargo test --test security_tests`)
- **Location**: `tests/security_tests.rs`
- **Coverage**: Security properties, encryption integrity, file permissions
- **Tests**: 11 tests covering crypto security, tamper detection, access controls
- **Purpose**: Ensure security guarantees are maintained

### Benchmark Tests (`cargo test --test benchmark_tests`)
- **Location**: `tests/benchmark_tests.rs`
- **Coverage**: Performance characteristics, memory usage, concurrency
- **Tests**: 5 tests covering performance under various conditions
- **Purpose**: Catch performance regressions (CI runs on stable Linux only)

## CI/CD Test Strategy

### Matrix Testing
- **Platforms**: Linux (Ubuntu), macOS, Windows
- **Rust Versions**: stable, beta (Linux + nightly)
- **Parallel execution** for faster feedback

### Test Execution Order
1. **Code Quality** - formatting, linting (fast feedback)
2. **Security Audit** - dependency vulnerabilities  
3. **Unit Tests** - core functionality (fastest)
4. **Integration Tests** - CLI behavior
5. **Security Tests** - crypto properties
6. **Benchmark Tests** - performance (Linux stable only)
7. **Coverage** - test coverage analysis
8. **Release Build** - multi-platform binaries

### Coverage Target
- **Unit Tests**: >95% line coverage
- **Integration Tests**: All CLI commands + error cases
- **Security Tests**: All crypto operations + edge cases

## Local Testing

```bash
# Run all tests
cargo test

# Run specific test suites  
cargo test --lib                    # Unit tests only
cargo test --test integration_tests # Integration tests only
cargo test --test security_tests    # Security tests only
cargo test --test benchmark_tests   # Benchmark tests only

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name -- --nocapture

# Run tests with environment
PUG_MASTER_PASSWORD=test123 cargo test
```

## Test Data Management
- **Temporary files**: All tests use `tempfile` crate for cleanup
- **Test isolation**: Each test uses unique file paths
- **Environment variables**: Tests set/cleanup their own env vars
- **No shared state**: Tests can run in parallel safely

## Performance Expectations
- **Unit tests**: < 10 seconds total
- **Integration tests**: < 30 seconds total  
- **Security tests**: < 20 seconds total
- **Benchmark tests**: < 60 seconds total (Argon2 is intentionally slow)

## Security Test Coverage
- ✅ Encryption/decryption round-trip
- ✅ Wrong password detection  
- ✅ File tampering detection
- ✅ File permission security (600)
- ✅ Nonce uniqueness per encryption
- ✅ Key derivation determinism
- ✅ Salt consistency
- ✅ Error handling for corrupted data
- ✅ Unicode/special character handling
- ✅ Large data handling