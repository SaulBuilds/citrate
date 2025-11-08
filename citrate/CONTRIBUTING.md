# Contributing to Citrate

Thank you for your interest in contributing to Citrate! This document provides guidelines and instructions for contributing to the project.

## Quick Start for Contributors

### Prerequisites

- Rust 1.75 or newer
- Git
- Basic development tools (gcc/clang, make, pkg-config)

### Building the Project

```bash
# Clone the repository
git clone https://github.com/SaulBuilds/citrate.git
cd citrate/citrate

# Build the entire workspace
cargo build --workspace --release

# Or build specific packages
cargo build -p citrate-node --release
cargo build -p citrate-wallet --release
```

**Important:** The standard build does **not** require downloading the AI model files. The genesis block is loaded from the blockchain database at runtime.

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific package
cargo test -p citrate-consensus
cargo test -p citrate-execution

# Run with output
cargo test -- --nocapture
```

### Code Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting without making changes
cargo fmt --all -- --check
```

### Linting

```bash
# Run Clippy linter
cargo clippy --all-targets --all-features

# Fail on warnings
cargo clippy --all-targets --all-features -- -D warnings
```

## Development Workflow

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/citrate.git
cd citrate/citrate

# Add upstream remote
git remote add upstream https://github.com/SaulBuilds/citrate.git
```

### 2. Create a Feature Branch

```bash
# Update main branch
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feature/your-feature-name
```

### 3. Make Changes

- Write clean, idiomatic Rust code
- Follow existing code style and conventions
- Add tests for new functionality
- Update documentation as needed

### 4. Test Your Changes

```bash
# Run tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Build in release mode
cargo build --workspace --release
```

### 5. Commit Changes

```bash
# Stage changes
git add .

# Commit with descriptive message
git commit -m "feat(module): Add new feature

- Detailed description of changes
- Why the change was needed
- Any breaking changes"
```

**Commit Message Format:**
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, etc.
- `perf`: Performance improvements

### 6. Push and Create Pull Request

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
```

## Project Structure

```
citrate/
├── core/                    # Core blockchain components
│   ├── consensus/          # GhostDAG consensus implementation
│   ├── execution/          # LVM execution engine
│   ├── storage/            # State and block storage
│   ├── network/            # P2P networking
│   ├── sequencer/          # Transaction sequencing
│   ├── api/                # JSON-RPC API
│   ├── mcp/                # Model Context Protocol
│   └── economics/          # Tokenomics and rewards
├── node/                    # Main blockchain node
├── wallet/                  # CLI wallet
├── gui/citrate-core/       # Tauri desktop application
├── contracts/               # Solidity smart contracts
├── sdk/                     # Official SDKs
└── docs/                    # Documentation

```

## Areas to Contribute

### High Priority

- **Performance Optimizations** - Improve block processing speed
- **Test Coverage** - Add unit and integration tests
- **Documentation** - Improve guides and API docs
- **Bug Fixes** - Fix reported issues

### Medium Priority

- **Feature Implementations** - Add new RPC methods, precompiles
- **Developer Tools** - Improve CLI tools and debugging
- **Smart Contracts** - Add governance contracts
- **SDKs** - Enhance JavaScript and Python SDKs

### Good First Issues

Look for issues labeled `good-first-issue` on GitHub:
- Documentation improvements
- Test additions
- Small bug fixes
- Code cleanup

## Coding Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` with default settings
- Avoid `unsafe` code unless absolutely necessary
- Prefer explicit error handling over `.unwrap()` or `.expect()`
- Add documentation comments for public APIs

### Documentation

- Add doc comments (`///`) for all public items
- Include examples in doc comments when helpful
- Update README.md and guides when adding features
- Keep comments concise and meaningful

### Testing

- Write unit tests for new functions
- Add integration tests for complex features
- Use property-based testing where applicable
- Aim for >80% code coverage

### Error Handling

```rust
// Good - explicit error handling
pub fn process_block(block: &Block) -> Result<Receipt, ExecutionError> {
    let validated = validate_block(block)?;
    execute_transactions(&validated)
}

// Avoid - using unwrap/expect
pub fn process_block(block: &Block) -> Receipt {
    let validated = validate_block(block).unwrap();
    execute_transactions(&validated).expect("execution failed")
}
```

### Async Code

- Use `tokio` runtime for async operations
- Prefer `async fn` over manual `Future` implementations
- Use `#[tokio::test]` for async tests
- Avoid blocking operations in async contexts

## Building with Features

Citrate uses Cargo feature flags for optional functionality:

### Standard Build (Default)
```bash
# No AI model embedding - fast builds for development
cargo build --workspace --release
```

### Genesis Creation (Maintainers Only)
```bash
# Only needed when creating a new genesis block
# Requires downloading the 417 MB BGE-M3 model first

# Download model
mkdir -p node/assets
cd node/assets
wget https://huggingface.co/BAAI/bge-m3-gguf/resolve/main/bge-m3-q4.gguf
cd ../..

# Build with model embedding
cargo build -p citrate-node --release --features embed-genesis-model
```

See [docs/guides/building-from-source.md](docs/guides/building-from-source.md) for details.

## Pull Request Guidelines

### Before Submitting

- [ ] Code builds without errors or warnings
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] Linter passes (`cargo clippy --all-targets --all-features`)
- [ ] Documentation is updated
- [ ] Commit messages follow conventions

### PR Description Template

```markdown
## Description
Brief description of the changes

## Motivation
Why is this change needed?

## Changes
- List of specific changes made
- File-by-file breakdown if extensive

## Testing
How was this tested?

## Breaking Changes
List any breaking changes and migration path

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Changelog updated (if applicable)
```

### Review Process

1. Maintainers will review your PR within 1-3 business days
2. Address any feedback or requested changes
3. Once approved, maintainers will merge

## Development Tools

### Recommended VS Code Extensions

- **rust-analyzer** - Rust language support
- **CodeLLDB** - Debugging support
- **Even Better TOML** - TOML syntax highlighting
- **EditorConfig** - Maintain consistent coding styles

### Useful Commands

```bash
# Watch mode - rebuild on file changes
cargo watch -x build

# Check project without building
cargo check

# Generate documentation
cargo doc --open

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench

# Update dependencies
cargo update
```

## Getting Help

- **Documentation:** [docs.citrate.ai](https://docs.citrate.ai)
- **GitHub Issues:** [github.com/SaulBuilds/citrate/issues](https://github.com/SaulBuilds/citrate/issues)
- **Discord:** [discord.gg/citrate](https://discord.gg/citrate)
- **Email:** dev@citrate.ai

## License

By contributing to Citrate, you agree that your contributions will be licensed under the MIT License.

## Code of Conduct

Please be respectful and constructive in all interactions. We are committed to providing a welcoming and inclusive environment for all contributors.

---

**Thank you for contributing to Citrate!** 🚀
