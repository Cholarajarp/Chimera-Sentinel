# From Vanguard to Sentinel — Architectural Evolution

> This document explains the relationship between **Chimera Vanguard** (prior
> research) and **Chimera Sentinel** (this project). Sentinel is a clean-room
> implementation; see [`NOTICE.md`](../NOTICE.md) for the formal disclosure.

---

## Why This Document Exists

The Google All Things Agentic Hackathon requires disclosure of prior work.
Chimera Vanguard was a research prototype exploring AI agent security concepts.
Sentinel builds on those *lessons learned* with a completely new implementation
targeting Google Cloud's managed agent services.

---

## Architectural Lessons Carried Forward

### 1. Immutable Agent Bill of Materials (ABOM)

**Vanguard Lesson**: Treating agents as mutable bundles of prompts, models, and
tools makes it impossible to reason about what was tested versus what is deployed.

**Sentinel Implementation**: Every candidate revision binds source digest, model
reference, prompt/config digest, tool manifest digest, Gateway policy digest,
Model Armor config digest, and Agent Identity into an immutable ABOM before
certification begins.

### 2. Deterministic Business-Logic Oracle

**Vanguard Lesson**: Asking an LLM "did the agent do the right thing?" is
circular. Model text should never be the safety oracle.

**Sentinel Implementation**: The Mock ERP Ledger Oracle provides deterministic
before/after snapshots. The Rust policy engine evaluates
`unauthorized_released_payments == 0` as a mathematical invariant — not a
model judgment.

### 3. Defense in Depth

**Vanguard Lesson**: No single defense layer (content filtering, prompt
engineering, or access control) is sufficient alone.

**Sentinel Implementation**: Four independent defense layers operate in series:
1. **Model Armor** — content inspection and prompt injection detection
2. **Agent Gateway** — identity-scoped tool authorization
3. **Ledger Oracle** — deterministic side-effect verification
4. **Rust Policy Engine** — final admission decision

### 4. Separation of AI Observation from Authorization

**Vanguard Lesson**: When the AI system that analyzes agent behavior can also
approve it, the security boundary collapses.

**Sentinel Implementation**: Python/Gemini produce typed, untrusted observations.
Rust computes the final admission gate. Python cannot approve, sign, or write
canonical decisions.

---

## What Changed Completely

| Dimension | Vanguard | Sentinel |
|---|---|---|
| **Primary Language** | Python | Rust (control plane, policy, evidence, attestation) |
| **Agent Framework** | Custom orchestration | Google ADK (official Python SDK) |
| **Cloud Platform** | Azure / OpenAI | Google Cloud (Vertex AI, Cloud Run, KMS) |
| **Model** | GPT-4 | Gemini 3.5+ on Vertex AI |
| **Identity Model** | Shared API keys | Google Agent Identity (per-principal) |
| **Content Safety** | Custom regex/heuristics | Google Model Armor |
| **Tool Gateway** | None | Google Agent Gateway |
| **Memory** | Custom RAG | Google Memory Bank |
| **Signing** | Local RSA keys | Google Cloud KMS (asymmetric) |
| **State Store** | SQLite | Google Cloud Firestore |
| **Evidence Store** | Local filesystem | Google Cloud Storage |
| **Observability** | Custom logging | OpenTelemetry + Cloud Trace |
| **Deployment** | Docker Compose only | Terraform + Cloud Run + Cloud Build |
| **UI** | Flask templates | Next.js enterprise console |

---

## Code Reuse: Zero

No Vanguard source code, prompts, test fixtures, configuration files, Docker
images, CI pipelines, or UI assets were copied, ported, translated, or adapted
into Sentinel. The implementation was written from scratch during the hackathon
eligibility period.

See [`NOTICE.md`](../NOTICE.md) for the complete clean-room declaration.
