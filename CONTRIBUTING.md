# Contributing to Chimera Sentinel

Thank you for your interest in contributing to Chimera Sentinel! This document
provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.92.0+ | Control plane, policy engine, CLI |
| Python | 3.11+ | ADK certifier |
| Node.js | v20+ | Web console |
| uv | Latest | Python package management |
| pnpm | Latest | Node.js package management |

### Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/Cholarajarp/Chimera-Sentinel.git
cd Chimera-Sentinel

# 2. Bootstrap all dependencies
make bootstrap

# 3. Copy environment configuration
cp .env.example .env
# Edit .env with your Google Cloud project values

# 4. Verify everything compiles
cargo check --workspace

# 5. Run tests
make test
```

## Code Standards

### Rust

- **Edition**: 2021
- **Unsafe code**: Forbidden (`#[forbid(unsafe_code)]`)
- **Linting**: `cargo clippy --workspace -- -D warnings`
- **Formatting**: `cargo fmt --all`
- **Documentation**: Public items should have doc comments

### Python

- **Linting**: `ruff check src/`
- **Formatting**: `ruff format src/`
- **Type checking**: `mypy src/`

### TypeScript / Next.js

- **Linting**: `pnpm lint`
- **Type checking**: `pnpm type-check`

## Testing

```bash
# Run all tests
make test

# Rust unit tests only
make unit

# Contract/schema tests
make contract

# Validate 80-case evaluation corpus
make corpus-validate

# Verify demo attestation offline
make verify-demo

# Full pre-submission checklist
make submission-check
```

## Pull Request Process

1. **Branch** from `main` with a descriptive name (`feat/...`, `fix/...`, `docs/...`)
2. **Write tests** for new functionality
3. **Ensure CI passes**: `make check lint test`
4. **Update documentation** if behavior changes
5. **Keep commits atomic** with clear messages
6. **Request review** from a maintainer

## Architecture Rules

These rules are non-negotiable to preserve the security model:

1. **Never move authorization or canonical state into Python** — Rust owns all
   admission decisions
2. **Never trust model-generated text as a safety proof** — use the deterministic
   ledger oracle
3. **Never disable fail-closed behavior** — infrastructure failures block admission
4. **Never commit secrets, credentials, or .env files**
5. **Never add `unsafe` code** — it is forbidden at the workspace level

## License

By contributing, you agree that your contributions will be licensed under the
Apache License 2.0.
