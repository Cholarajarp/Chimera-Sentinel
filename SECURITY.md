# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Chimera Sentinel, please report it
responsibly. **Do not open a public GitHub issue.**

### How to Report

1. **Email**: Send a detailed report to the project maintainers via the repository
   contact information
2. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Resolution Target**: Within 30 days for critical issues

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | ✅ Current release |

## Security Design Principles

Chimera Sentinel is built on the following security principles:

### 1. Rust Decision Authority
All authorization and admission decisions are computed in pure Rust. Gemini and
Python ADK produce untrusted typed observations only — they cannot approve,
promote, or sign attestations.

### 2. Deterministic Side-Effect Oracle
Model-generated text is never the safety oracle. Business invariants are verified
through deterministic ledger snapshot comparisons (e.g.,
`unauthorized_released_payments == 0`).

### 3. Identity Separation
Candidate agents, the ADK certifier, and the control plane operate under distinct
Google Cloud Workload Identity principals with least-privilege IAM bindings.

### 4. Content-Addressed Evidence
All evidence objects are content-addressed with SHA-256 digests. Evidence manifests
are immutable after finalization. KMS-signed attestations bind all digests
cryptographically.

### 5. Telemetry Redaction
Strict allowlist-based redaction prevents raw invoice bodies, prompts, tokens,
canary secrets, or PII from exporting to Cloud Trace or any observability sink.

### 6. Fail-Closed Design
Infrastructure failures, incomplete evidence, and managed service unavailability
all result in blocked admission — never silent fallback or degraded-mode approval.

## Known Limitations

- This is a hackathon project and has not undergone formal security audit
- The evaluation corpus is synthetic and does not claim universal agent safety
- Cloud KMS key management follows Google Cloud best practices but has not been
  independently certified
- The system prevents tested attack categories; novel attack vectors may exist

## Dependency Security

- Rust dependencies are pinned in `Cargo.lock` and audited via `cargo audit`
- Python dependencies are pinned in `uv.lock` and audited via `pip-audit`
- Node.js dependencies are pinned in `pnpm-lock.yaml` and audited via `pnpm audit`
- Container images use minimal base images and run as non-root
