# Contributing to Pump Sniper

Thank you for your interest in contributing to Pump Sniper! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## How to Contribute

### Reporting Issues

Before creating an issue, please check if it already exists. When creating a new issue, include:

- Clear, descriptive title
- Detailed description of the problem
- Steps to reproduce the issue
- Expected vs actual behavior
- System information (OS, Rust version, Solana CLI version)
- Relevant logs or error messages

### Suggesting Enhancements

Enhancement suggestions are welcome! Please provide:

- Clear use case explanation
- Detailed description of proposed solution
- Alternative solutions considered
- Potential impact on existing functionality

### Pull Requests

1. **Fork the Repository**
   ```bash
   git clone https://github.com/kernlog/pump-sniper.git
   cd pump-sniper
   git remote add upstream https://github.com/kernlog/pump-sniper.git
   ```

2. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Your Changes**
   - Follow existing code style and conventions
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass

4. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```
   
   Follow conventional commit format:
   - `feat:` New feature
   - `fix:` Bug fix
   - `docs:` Documentation changes
   - `test:` Test additions or changes
   - `refactor:` Code refactoring
   - `perf:` Performance improvements
   - `chore:` Maintenance tasks

5. **Push to Your Fork**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request**
   - Provide clear PR title and description
   - Reference any related issues
   - Ensure CI checks pass

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Solana CLI tools
- Git

### Building from Source

```bash
# Clone the repository
git clone https://github.com/kernlog/pump-sniper.git
cd pump-sniper

# Build the project
cargo build

# Run tests
cargo test

# Build release version
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Style

- Use `rustfmt` for formatting:
  ```bash
  cargo fmt
  ```

- Use `clippy` for linting:
  ```bash
  cargo clippy -- -D warnings
  ```

## Project Structure

```
pump-sniper/
├── src/
│   ├── accounts/       # Solana account structures
│   ├── bin/           # Binary executables
│   ├── common/        # Shared components
│   ├── constants/     # Program constants
│   ├── error/         # Error handling
│   ├── instructions/  # Transaction builders
│   └── utils/         # Helper functions
├── tests/             # Integration tests
└── examples/          # Usage examples
```

## Testing Guidelines

- Write unit tests for new functions
- Add integration tests for new features
- Ensure tests are deterministic
- Mock external dependencies when possible
- Test edge cases and error conditions

## Documentation

- Add inline documentation for public APIs
- Update README for user-facing changes
- Include examples for complex features
- Keep CHANGELOG.md updated

## Security

- Never commit sensitive data (keys, passwords)
- Review dependencies for vulnerabilities
- Follow Solana security best practices
- Report security issues privately to maintainers

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create pull request for release
4. After merge, tag release:
   ```bash
   git tag -a v0.x.x -m "Release version 0.x.x"
   git push origin v0.x.x
   ```

## Getting Help

- Check existing documentation
- Search closed issues
- Join project discussions
- Ask questions in issues

## Recognition

Contributors will be recognized in:
- GitHub contributors page
- Release notes
- Project documentation

Thank you for contributing to Pump Sniper!