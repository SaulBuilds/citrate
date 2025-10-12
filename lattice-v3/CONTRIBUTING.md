# Contributing to Lattice

Thank you for your interest in contributing to Lattice! This document provides guidelines and information for contributors.

## Code of Conduct

We are committed to providing a welcoming and inspiring community for all. Please read and follow our Code of Conduct:

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Respect differing viewpoints and experiences

## Getting Started

### Prerequisites

1. **Development Environment**
   - macOS 13+ with Apple Silicon (M1/M2/M3) recommended
   - Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
   - Python 3.9+ for AI tools
   - Node.js 18+ for GUI development

2. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/lattice-v3.git
   cd lattice-v3
   ```

3. **Build and Test**
   ```bash
   cargo build --release
   cargo test --workspace
   ```

## Development Process

### 1. Find an Issue

- Check [open issues](https://github.com/lattice/lattice-v3/issues)
- Look for `good first issue` or `help wanted` labels
- Comment on the issue to claim it
- If proposing new features, open an issue first for discussion

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
```

### 3. Make Changes

Follow our coding standards:

#### Rust Code
- Follow standard Rust conventions
- Use `cargo fmt` before committing
- Ensure `cargo clippy` passes
- Add tests for new functionality
- Document public APIs with doc comments

#### Python Code (AI Tools)
- Follow PEP 8 style guide
- Use type hints where appropriate
- Add docstrings to functions
- Test with multiple Python versions

#### Solidity Contracts
- Follow Solidity style guide
- Include NatSpec comments
- Add comprehensive tests
- Consider gas optimization

### 4. Write Tests

- Unit tests for individual components
- Integration tests for feature interactions
- Document test scenarios
- Aim for >80% code coverage

### 5. Update Documentation

- Update README if adding features
- Add inline documentation
- Update API docs if changing interfaces
- Include examples for new functionality

### 6. Commit Guidelines

We follow conventional commits:

```bash
# Format: <type>(<scope>): <subject>

feat(consensus): add parallel block validation
fix(api): resolve memory leak in RPC handler
docs(readme): update installation instructions
test(execution): add EVM compatibility tests
refactor(storage): optimize IPFS chunking
perf(network): improve p2p message throughput
chore(deps): update Rust dependencies
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions or changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

### 7. Submit Pull Request

1. Push your branch
   ```bash
   git push origin feature/your-feature-name
   ```

2. Open PR with:
   - Clear title and description
   - Reference related issues
   - List key changes
   - Include test results
   - Add screenshots if UI changes

3. PR Template:
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing
   - [ ] Unit tests pass
   - [ ] Integration tests pass
   - [ ] Manual testing completed

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Documentation updated
   - [ ] Tests added/updated
   - [ ] No new warnings

   Fixes #(issue)
   ```

## Project Structure

### Core Components

- **consensus/**: GhostDAG consensus implementation
  - Focus on correctness and performance
  - Extensive testing required

- **execution/**: EVM execution environment
  - Maintain EVM compatibility
  - Test with standard contracts

- **storage/**: State and block storage
  - Optimize for SSD performance
  - Consider memory usage

- **network/**: P2P networking
  - Test with various network conditions
  - Ensure message propagation

### AI Components

- **mcp/**: Model Context Protocol
  - Follow MCP specification
  - Ensure model compatibility

- **tools/**: Model conversion tools
  - Support multiple frameworks
  - Optimize for Apple Silicon

## Testing

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific module
cargo test -p lattice-consensus

# With output
cargo test -- --nocapture

# Integration tests
./scripts/run_integration_tests.sh

# AI pipeline tests
./tests/test_ai_pipeline.sh
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = prepare_input();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_feature() {
        // Async test implementation
    }
}
```

## Performance Considerations

- Profile before optimizing
- Benchmark critical paths
- Consider memory allocation
- Test on different hardware
- Document performance characteristics

## Security

- Never commit secrets or keys
- Validate all inputs
- Use safe Rust patterns
- Review dependencies
- Report vulnerabilities to security@lattice.xyz

## Documentation

### Code Documentation

```rust
/// Processes a block through the GhostDAG algorithm.
///
/// # Arguments
///
/// * `block` - The block to process
/// * `state` - Current DAG state
///
/// # Returns
///
/// Returns `Ok(ProcessedBlock)` on success, or an error.
///
/// # Example
///
/// ```
/// let processed = process_block(block, &mut state)?;
/// ```
pub fn process_block(block: Block, state: &mut DagState) -> Result<ProcessedBlock> {
    // Implementation
}
```

### API Documentation

- Document all public APIs
- Include request/response examples
- List error conditions
- Provide usage scenarios

## Review Process

1. **Automated Checks**
   - CI/CD runs tests
   - Linting and formatting
   - Security scanning
   - Performance benchmarks

2. **Code Review**
   - At least one maintainer review
   - Address all feedback
   - Resolve conversations
   - Update based on suggestions

3. **Merge Requirements**
   - All tests passing
   - Documentation updated
   - No merge conflicts
   - Approved by maintainer

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **Discord**: [discord.gg/lattice](https://discord.gg/lattice)
- **Forum**: [forum.lattice.xyz](https://forum.lattice.xyz)

### Getting Help

- Read existing documentation
- Search closed issues
- Ask in Discord #dev-help
- Tag maintainers if blocked

## Recognition

Contributors are recognized in:
- Release notes
- Contributors file
- Project website
- Community calls

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Feel free to:
- Open an issue for clarification
- Ask in Discord
- Email: contributors@lattice.xyz

Thank you for contributing to Lattice! 🚀