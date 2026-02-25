# Contributing to eche-lite

Thank you for your interest in contributing to eche-lite! This document provides guidelines and workflows for contributing.

## Getting Started

1. Fork the repository and clone your fork
2. Create a feature branch from `main`
3. Make your changes
4. Submit a pull request

## Development Setup

eche-lite is a Rust crate targeting both `std` and `no_std` environments.

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/defenseunicorns/eche-lite.git
cd eche-lite
cargo build
```

## Pre-Commit Checks

Before submitting a PR, ensure all of the following pass locally:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --no-default-features
```

The CI pipeline runs these same checks on every PR.

## Branching Strategy

We use **trunk-based development** on `main` with short-lived feature branches:

- Branch from `main` for all changes
- Keep branches small and focused (prefer multiple small PRs over one large one)
- Squash-and-merge to `main`

## Commit Requirements

- **GPG-signed commits are required.** Configure commit signing per [GitHub's documentation](https://docs.github.com/en/authentication/managing-commit-signature-verification).
- Write clear, descriptive commit messages

## Pull Request Process

1. Open a PR against `main` with a clear description of the change
2. Fill out the PR template
3. Ensure CI passes (fmt, clippy, tests, no_std build)
4. PRs require **1 approving review** from a CODEOWNERS member
5. External contributors require **2 approving reviews**
6. PRs are squash-merged to maintain a clean history

## Architectural Changes

For significant architectural changes, open an issue first to discuss the approach. Reference the relevant ADR (Architecture Decision Record) if one exists, or propose a new one.

## Reporting Issues

Use GitHub Issues to report bugs or request features. Please use the provided issue templates.

## Code of Conduct

All contributors are expected to follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

By contributing, you agree that your contributions will be licensed under the [Apache License 2.0](LICENSE).
