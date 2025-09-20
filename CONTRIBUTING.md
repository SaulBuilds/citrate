# Contributing to Lattice Network (v3)

Thanks for your interest in contributing! This document explains how to propose changes, report issues, and get your work merged.

## Ways To Contribute
- Report bugs and request features via GitHub Issues (include steps to reproduce and environment details)
- Improve documentation (README, docs-portal) and examples
- Send pull requests for code fixes, tests, or new capabilities aligned with the roadmap

## Development Setup
- Rust (stable) + Cargo
- Node.js >= 20 + npm
- Optional: Foundry (forge) for Solidity tests

Quick build checks

```
# Rust workspace
cd lattice-v3 && cargo fmt --all && cargo clippy --all-targets --all-features -D warnings && cargo test --workspace

# GUI (Tauri) / Explorer / Docs (only build the ones you changed)
cd lattice-v3/gui/lattice-core && npm ci && npm run build
cd ../../explorer && npm ci && npm run build
cd ../../../docs-portal && npm install && npm run build

# SDK (JavaScript/TypeScript)
cd lattice-v3/sdk/javascript && npm install && npm run build && npm test

# Contracts (optional, if touched)
cd lattice-v3 && forge test -vv
```

## Branching And Commits
- Branch names: `feat/*`, `fix/*`, `docs/*`, `ci/*`, `refactor/*`
- Use Conventional Commits for messages:
  - `feat(consensus): add tie-breaker for equal blue score`
  - `fix(api): handle invalid address in eth_call`
- Keep commits focused and descriptive; avoid large mixed changes when possible

## Pull Request Guidelines
- Small, focused PRs are easier to review and merge
- Open an Issue first for large changes to discuss scope and design
- Checklist before opening a PR:
  - [ ] Code builds locally (Rust/JS as applicable)
  - [ ] Tests pass; new tests added for new behavior/bug fixes
  - [ ] Docs updated (README, docs-portal, comments) where relevant
  - [ ] No noisy artifacts or lockfiles unintentionally changed
  - [ ] Follows style: `cargo fmt` + `cargo clippy -D warnings`; TS/JS builds clean
- CI must be green; at least one maintainer approval is required

## Coding Style & Quality
- Rust: idiomatic code, no `unsafe` unless justified; lint with clippy; prefer small modules and clear error types
- TypeScript/React: strict typing, avoid `any`, keep components small and testable; ensure production builds are clean
- Solidity (Foundry): small, testable contracts; add unit/fuzz tests where appropriate
- Logging: use structured logging (`tracing` in Rust); avoid noisy `println!` in hot paths

## Testing
- Prefer unit tests for critical logic; integration tests for module boundaries; end‑to‑end where valuable
- Reproduce bugs with tests and keep them in the tree to prevent regressions
- Contracts: use forge tests; record gas snapshots when relevant

## Security
- Do not open public issues for vulnerabilities. Please report privately to security@lattice.example (replace with the correct project contact when available)
- Provide enough information for reproduction and impact assessment

## Licensing
- By contributing, you agree your work is licensed under the repository’s MIT license
- If your organization requires DCO sign‑off, include: `Signed-off-by: Full Name <email>`

## Releases
- Maintainers cut releases by tagging `v*` (e.g., `v0.1.0`)
- CI builds and publishes Node/CLI binaries and GUI installers to GitHub Releases

Thanks again for helping make Lattice better!

