# Contributing to Ultra-Secure Drone Swarm Communication System

First off, thank you for considering contributing to this project! It's people like you that make this drone swarm system better for everyone.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Security Considerations](#security-considerations)

## Code of Conduct

This project and everyone participating in it is governed by our commitment to foster an open and welcoming environment. By participating, you are expected to:

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When you create a bug report, include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples** (code snippets, configuration files)
- **Describe the behavior you observed** and what you expected
- **Include your environment details** (OS, Rust version, hardware)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion:

- **Use a clear and descriptive title**
- **Provide a detailed description** of the suggested enhancement
- **Explain why this enhancement would be useful**
- **List any alternatives** you've considered

### Contributing Code

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following our coding standards
3. **Add tests** for any new functionality
4. **Ensure all tests pass** (`cargo test`)
5. **Update documentation** as needed
6. **Submit a pull request**

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Cargo
- Git

### Setup Steps

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/drone-swarm-system.git
cd drone-swarm-system

# Add upstream remote
git remote add upstream https://github.com/mahii6991/drone-swarm-system.git

# Create a feature branch
git checkout -b feature/your-feature-name

# Install dependencies and build
cargo build

# Run tests
cargo test
```

### Building and Testing

```bash
# Build the project
cargo build --release

# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Check code formatting
cargo fmt -- --check

# Run clippy linter
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

## Pull Request Process

1. **Update the README.md** with details of changes if applicable
2. **Add tests** for new functionality
3. **Ensure CI/CD passes** (all tests, formatting, clippy)
4. **Update documentation** including inline comments and doc comments
5. **Follow the commit message guidelines** (see below)
6. **Request review** from maintainers
7. **Address review feedback** promptly

### Commit Message Guidelines

We follow conventional commits format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(consensus): add support for dynamic cluster membership

Implements the ability to add/remove nodes from the consensus
cluster without downtime.

Closes #123
```

```
fix(crypto): resolve nonce collision in high-frequency scenarios

The previous implementation could generate duplicate nonces when
processing more than 1000 messages per second.
```

## Coding Standards

### Rust Style Guide

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `cargo fmt` for automatic formatting
- Run `cargo clippy` and fix all warnings
- Aim for 100% safe Rust (no `unsafe` blocks without justification)

### Code Quality

- **Keep functions small and focused** (ideally < 50 lines)
- **Write descriptive variable names**
- **Add doc comments** for public APIs
- **Use type system** to enforce invariants
- **Prefer composition** over inheritance
- **Avoid premature optimization**

### Documentation

All public APIs must have documentation:

```rust
/// Encrypts a message using ChaCha20-Poly1305 AEAD.
///
/// # Arguments
///
/// * `plaintext` - The message to encrypt
/// * `key` - The encryption key (32 bytes)
/// * `nonce` - Unique nonce (12 bytes)
///
/// # Returns
///
/// Encrypted ciphertext with authentication tag
///
/// # Examples
///
/// ```
/// let ciphertext = encrypt(&message, &key, &nonce);
/// ```
pub fn encrypt(plaintext: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Vec<u8> {
    // Implementation
}
```

## Testing Guidelines

### Test Coverage

- **Unit tests** for individual functions and modules
- **Integration tests** for component interactions
- **Property-based tests** for cryptographic functions
- Aim for **>80% code coverage**

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"Hello, World!";

        let ciphertext = encrypt(plaintext, &key, &nonce);
        let decrypted = decrypt(&ciphertext, &key, &nonce).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    #[should_panic(expected = "Invalid key length")]
    fn test_encryption_with_invalid_key() {
        let invalid_key = [0u8; 16]; // Wrong size
        let nonce = [0u8; 12];
        encrypt(b"test", &invalid_key, &nonce);
    }
}
```

### Test Organization

- Place unit tests in the same file as the code (`#[cfg(test)] mod tests`)
- Place integration tests in the `tests/` directory
- Use descriptive test names that explain what is being tested

## Security Considerations

This is a security-critical project. Please follow these guidelines:

### Security Best Practices

1. **Never commit secrets** (keys, passwords, tokens)
2. **Use constant-time comparisons** for cryptographic operations
3. **Validate all inputs** at trust boundaries
4. **Handle errors securely** (no information leakage)
5. **Document security assumptions** in code

### Reporting Security Vulnerabilities

**DO NOT** open public issues for security vulnerabilities. Instead:

1. Email the maintainer privately (include "SECURITY" in subject)
2. Provide detailed description of the vulnerability
3. Include steps to reproduce (if applicable)
4. Wait for acknowledgment before public disclosure

We will respond within 48 hours and work with you to address the issue.

### Cryptographic Changes

Any changes to cryptographic code require:

- Justification with references to security research
- Review by someone with cryptography expertise
- Additional testing and validation
- Documentation of security properties

## Areas for Contribution

We especially welcome contributions in these areas:

### High Priority

- [ ] Hardware driver implementations (GPS, IMU, radio modules)
- [ ] Additional consensus algorithms
- [ ] Performance optimizations
- [ ] Embedded platform support (STM32, ESP32)
- [ ] Documentation improvements

### Medium Priority

- [ ] Additional formation patterns
- [ ] Visualization tools
- [ ] Simulation environments
- [ ] Example applications
- [ ] Benchmarking suite

### Research Areas

- [ ] Post-quantum cryptography integration
- [ ] Advanced swarm algorithms
- [ ] Machine learning optimizations
- [ ] Energy efficiency improvements

## Getting Help

If you need help:

- Check the [documentation](https://github.com/mahii6991/drone-swarm-system)
- Review existing [issues](https://github.com/mahii6991/drone-swarm-system/issues)
- Ask questions in [discussions](https://github.com/mahii6991/drone-swarm-system/discussions)
- Join our community channels (if applicable)

## Recognition

Contributors will be recognized in:

- The project README (Contributors section)
- Release notes for significant contributions
- GitHub contributors page

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0, the same license as this project.

---

Thank you for contributing to making drone swarm systems safer and more capable!
