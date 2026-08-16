# All Things Agentic Hackathon Checklist

## Submission target

- **Track:** Fortified Enterprise Fleet
- **Deadline:** August 31, 2026 at 5:00 PM PDT
- **Project:** Chimera Sentinel
- **Rule posture:** new clean-room implementation; Chimera Vanguard disclosed as prior research

Statuses: `NOT_STARTED`, `IN_PROGRESS`, `BLOCKED`, `VERIFIED`. A row is `VERIFIED` only when its artifact is retrievable and the proof has been rerun after the final deployment.

## Required technology evidence matrix

| Requirement | Implementation proof | Demo proof | Submission artifact | Owner | Status | Fail condition |
|---|---|---|---|---|---|---|
| Gemini 3.5 or newer | Explicit eligible model resource/version in ADK config and evidence | Live case execution and sanitized usage/trace | Dependency/config digest and run evidence | TBD | NOT_STARTED | Model not eligible, implicit alias, or only mock output |
| Google ADK | Pinned official ADK dependency and typed certifier implementation | Show live ADK-managed certification operation | Lockfile, source path, run metadata | TBD | NOT_STARTED | Generic Python call mislabeled as ADK |
| Google Cloud infrastructure | Cloud Run plus durable Cloud services provisioned with IAM | Hosted URL and selected Cloud console/resource evidence | Terraform/setup scripts and deployment evidence JSON | TBD | NOT_STARTED | Local-only project or unverifiable placeholders |
| Agent Registry | Candidate/tool registration or discovery bound to candidate manifest | Show exact registry resource on revision | Sanitized resource reference and fetch evidence | TBD | NOT_STARTED | Decorative badge or mutable/unbound reference |
| Long-running Agent Runtime | Durable asynchronous operation with lifecycle state | Start run, disconnect/reconnect, show continued execution | Runtime operation reference and timeline | TBD | NOT_STARTED | Synchronous request merely labeled long-running |
| Memory Bank | Approved prior disposition retrieved as advisory context | Show memory evidence and authorization-neutral ablation | Retrieval reference, write governance and ablation report | TBD | NOT_STARTED | Memory grants permission or is not actually called |
| Agent Identity | Separate candidate, certifier and control identities | Show exact candidate principal on tool attempt | IAM/resource evidence and negative impersonation test | TBD | NOT_STARTED | Shared broad credential or identity inferred from UI |
| Agent Gateway | Identity-scoped tool policy with allow/deny evidence | Valid draft allowed; `release_payment` denied | Policy digest, provider decision and ledger evidence | TBD | NOT_STARTED | Candidate can bypass Gateway or denial is simulated |
| Model Armor | Live content inspection configuration and disposition | Direct hostile case receives visible live disposition | Config digest and provider evidence reference | TBD | NOT_STARTED | Local regex/model claim represented as Model Armor |
| OpenTelemetry-compatible observability | W3C trace propagation and OTel exporters | Open one correlated sanitized Cloud trace | Trace ID, dashboard and redaction test | TBD | NOT_STARTED | Screenshots only, broken correlation, or sensitive payload |

If current official product terminology differs, update the row to the official name while preserving the required capability and proof. Do not invent compatibility.

## Flagship capability evidence

| Capability | Implementation proof | Demo proof | Artifact | Owner | Status | Fail condition |
|---|---|---|---|---|---|---|
| Immutable candidate/ABOM | Bound source/model/prompt/tools/identity/policy/config digests | Revision detail and comparison | Candidate JSON/schema | TBD | NOT_STARTED | Mutable or missing security-critical input |
| Durable idempotent workflow | Firestore state, command IDs, leases, optimistic concurrency | Refresh/reconnect; run continues | Restart/duplicate test report | TBD | NOT_STARTED | Process-local state or duplicate effects |
| Deterministic ledger oracle | Append-only ledger and Rust invariants | Released-payment delta remains zero | Before/after hashes and oracle output | TBD | NOT_STARTED | Model text used as safety proof |
| Rust deterministic gate | Versioned rules and structured explanation | `CONSTRAINED_APPROVAL_REQUIRED` with rule trace | Policy version/tests | TBD | NOT_STARTED | Python/Gemini sets final decision |
| Human constrained approval | RBAC, scope, expiry, audit and permission reduction | Reviewer removes release capability | Approval evidence | TBD | NOT_STARTED | Broadening, self-bypass, or cosmetic approval |
| Positive and negative retest | Safe draft plus forbidden release after policy update | Both results visible | Case evidence | TBD | NOT_STARTED | Sign before complete retest |
| Content-addressed evidence | Manifest and verified object hashes | Open evidence summary | Exported sanitized bundle | TBD | NOT_STARTED | Missing object or browser-local report |
| KMS-signed attestation | Canonical payload, asymmetric KMS signature and verifier | Valid verify then prepared tamper rejection | Envelope, public verification material, test vectors | TBD | NOT_STARTED | Local test key represented as Cloud KMS or unverifiable signature |
| Expiry/revocation | Verifier checks validity/revocation | Show status change/rejection | Revocation event/test | TBD | NOT_STARTED | Expired/revoked statement still authorizes |
| Product-truth labels | Enum across schemas/storage/UI | Point out `LIVE` and `REPLAY` states | Schema/UI tests | TBD | NOT_STARTED | Silent fallback or ambiguous synthetic data |

## Judging rubric alignment

### Innovation and operational utility — 40%

- [ ] Explain identity/revision/policy/evaluation-bound attestation as the central innovation.
- [ ] Show a real enterprise outcome: forbidden payment prevented while useful drafting remains.
- [ ] Demonstrate defense in depth rather than relying on one prompt detector.
- [ ] Quantify evaluation with exact fractions, safe utility, latency, cost and limitations.
- [ ] Show enterprise path: ABOM, policy packs, CI admission, drift, revocation and incident evidence without claiming roadmap items are live.

### Architectural discipline — 30%

- [ ] Rust trusted control plane and narrow typed Python ADK boundary are visible.
- [ ] Durable async state, idempotency, fail-closed behavior and deterministic oracle are tested.
- [ ] Candidate/certifier/control identities and least-privilege Gateway policy are distinct.
- [ ] Evidence is content-addressed; attestation is independently verifiable.
- [ ] Memory is advisory; telemetry is redacted; local/replay cannot masquerade as live.
- [ ] Architecture diagram matches deployed reality.

### Demo and production readiness — 30%

- [ ] Hosted end-to-end path works from a clean session.
- [ ] Demo stays near four minutes and follows `EVALUATION_AND_DEMO.md`.
- [ ] One correlated Google Cloud trace is legible.
- [ ] README deployment is reproducible with pinned dependencies.
- [ ] Tests/scans are green and no critical/high unresolved security issue remains.
- [ ] Backup replay is prepared and visibly labeled.

## Repository and reproducibility checklist

### Repository

- [ ] Public or judge-accessible repository at final immutable commit/tag.
- [ ] New eligible-period history; no Vanguard remote/submodule/copied source.
- [ ] `README.md`, license, `NOTICE.md`, `SECURITY.md`, contribution guide.
- [ ] Prior-work clean-room disclosure and limitations.
- [ ] Architecture diagram and service/data-flow explanation.
- [ ] Exact prerequisites, cost implications, quota/preview caveats and region.
- [ ] `.env.example` contains names only; secret scan is clean.
- [ ] Rust/Python/Node/Terraform/container/action versions and lockfiles pinned.
- [ ] Third-party assets, corpus sources and dependencies attributed/licensed.
- [ ] No generated build/cache/virtualenv/state/evidence secrets committed.

### Setup

- [ ] One documented bootstrap command after prerequisites.
- [ ] Google project/API/identity/IAM setup is reproducible and least privileged.
- [ ] Terraform handles stable resources; evolving resources use idempotent verified scripts.
- [ ] Clean deploy emits verified service/resource identifiers.
- [ ] Seed command creates synthetic data only.
- [ ] Live smoke command fails instead of falling back.
- [ ] Teardown and estimated cost/budget controls documented.
- [ ] Clean-environment reproduction performed by a second operator if possible.

### Validation

- [ ] Format, lint, type checks, unit, contracts, integration and builds pass.
- [ ] Playwright/accessibility flows pass.
- [ ] Full local corpus and final live corpus reports exist.
- [ ] Restart, duplicate/out-of-order, bypass, cross-tenant and failure tests pass.
- [ ] Evidence tamper, wrong scope, expiry and revocation reject.
- [ ] Secret/SAST/SCA/license/container/IaC scans pass policy.
- [ ] Telemetry canary proves no secret/raw prompt/invoice leakage.

## README checklist

The first screen should answer:

- [ ] What Sentinel does in one sentence.
- [ ] Why the exact-revision attestation matters.
- [ ] Which workflow is live.
- [ ] Which Google services are used and for what.
- [ ] Architecture image/diagram.
- [ ] 3–5 minute quick start or clear deployment path.
- [ ] Hosted demo and video links.
- [ ] Evaluation headline with denominator and limitations.
- [ ] Clean-room Vanguard disclosure.

Further sections:

- [ ] Threat model and trust boundaries.
- [ ] Rust versus Python responsibility table.
- [ ] Data provenance labels and no-fallback policy.
- [ ] Local versus live modes.
- [ ] Evidence/attestation verification command.
- [ ] Security/privacy and synthetic-data statement.
- [ ] Test/development commands.
- [ ] Infrastructure setup and teardown.
- [ ] Current capabilities versus roadmap/R&D.
- [ ] Known limitations and responsible-use language.

## Architecture diagram acceptance

- [ ] Shows user/console, Rust API/worker, Python ADK, managed agent services, candidate, Gateway, MCP/ledger, Firestore, Storage, KMS and OTel.
- [ ] Trust boundaries and identities are understandable.
- [ ] Asynchronous flow and durable state are visible.
- [ ] Final decision/signature authority is Rust/KMS, not Gemini.
- [ ] Diagram uses deployed service names and contains no planned-only component without a roadmap label.
- [ ] Export is readable in repository and video.

## Hosted-project preflight

- [ ] HTTPS and production authentication work; no hardcoded demo bypass.
- [ ] Logged-out/unauthorized behavior is safe.
- [ ] Non-root immutable revisions are deployed.
- [ ] Candidate cannot directly access control data, KMS, oracle, or bypass Gateway.
- [ ] Budgets, quotas, min/max instances, timeouts and retention are set.
- [ ] Health/readiness and dashboards are green.
- [ ] Synthetic seed is reset and deterministic.
- [ ] All URLs are final, tested and not placeholders.
- [ ] Browser developer console/network shows no secrets or URL tokens.
- [ ] Mobile/narrow and keyboard path is usable.

## Four-minute video checklist

- [ ] English narration and accurate captions.
- [ ] Approximately four minutes; no long waits or illegible console scrolling.
- [ ] Problem and exact differentiator in first 25 seconds.
- [ ] Visible Google Cloud execution/resources.
- [ ] Safe task, direct injection, obfuscated attempt, Gateway denial and zero ledger.
- [ ] Rust constrained decision, human narrowing and retest.
- [ ] KMS signature verification plus tamper rejection.
- [ ] Correlated trace without sensitive payload.
- [ ] Exact evaluation result and honest limitation.
- [ ] 1080p/readable audio and text.
- [ ] No notification, account, billing, credential or personal-data exposure.
- [ ] Video link and permissions verified while logged out.

## Devpost description outline

### One-line pitch

> Chimera Sentinel is an enterprise admission controller that makes an exact AI-agent revision earn production authority through adversarial testing, identity-scoped tool enforcement, deterministic side-effect evidence, human governance, and an expiring KMS-signed attestation.

### Problem

Agents are promoted as mutable bundles of models, prompts, identities, memory and tools. Enterprises lack release evidence tied to the exact deployable authority.

### Solution

Describe quarantine, durable certification, Model Armor, identity/Gateway, ledger oracle, advisory memory, deterministic Rust decision, constrained approval/retest and signed attestation.

### How it uses Google Cloud

Name only live integrations and link each to the evidence matrix. State why ADK/Gemini assists analysis while Rust owns authorization.

### Innovation

Emphasize attestations binding agent identity and complete security configuration, plus preservation of useful behavior through least-privilege retesting.

### Evaluation

Provide exact numerator/denominator, corpus version, safe-task result, holdout method, latency/cost and limitations.

### Built during hackathon

Include clean-room disclosure and concise prior Vanguard research statement.

### Future

Present ABOM fleet governance, policy packs, CI admission, drift/recertification, MCP supply chain, incidents and enterprise administration as roadmap—not live features.

## Required submission assets

| Asset | Link/path | Owner | Status | Validation |
|---|---|---|---|---|
| Hosted project | TBD | TBD | NOT_STARTED | Logged-out clean session |
| Source repository | TBD | TBD | NOT_STARTED | Judge-accessible final revision |
| README/setup | `README.md` | TBD | NOT_STARTED | Clean reproduction |
| Architecture diagram | `docs/ARCHITECTURE.md` plus exported image | TBD | IN_PROGRESS | Matches deployment |
| English description | Devpost draft TBD | TBD | NOT_STARTED | Claims/evidence review |
| Approximately four-minute video | TBD | TBD | NOT_STARTED | Permissions/captions/timing |
| Google Cloud execution proof | Deployment evidence/trace TBD | TBD | NOT_STARTED | Resource references open safely |
| Evaluation report | Generated path TBD | TBD | NOT_STARTED | Recomputed from raw results |
| Attestation sample/verifier | TBD | TBD | NOT_STARTED | Valid/tamper tests |
| License/provenance | `LICENSE`, `NOTICE.md`, lineage docs | TBD | IN_PROGRESS | License/provenance scan |

## Optional bonus work — only after G5

- [ ] Short technical article explaining identity-bound attestations.
- [ ] Public social post/video clip with no inflated claims.
- [ ] Additional safe model/provider integration if rules reward it and architecture remains truthful.
- [ ] Second synthetic vertical (customer-support refund or CI deployment agent).
- [ ] Offline verifier package/release.
- [ ] Policy-pack example mapped to OWASP GenAI/NIST AI RMF with clear “evidence mapping, not compliance” language.

Optional work must not endanger the flagship, submission quality, or security.

## Final freeze procedure

### August 29 — code freeze

- [ ] Freeze commit/image/config/policy/corpus digests.
- [ ] Run final development corpus and all validation/scans.
- [ ] Deploy immutable final revisions.
- [ ] Resolve only blockers after freeze and record every change.

### August 30 — evidence and recording

- [ ] Run declared live workflow and holdout according to protocol.
- [ ] Export sanitized evidence/results and verify attestation.
- [ ] Record and upload video; perform two timed rehearsals.
- [ ] Execute setup as a clean operator.

### August 31 — submit early

- [ ] Recheck deadline timezone: 5:00 PM PDT.
- [ ] Verify hosted URL, repository and video from logged-out browser.
- [ ] Confirm all Devpost fields saved and links resolve.
- [ ] Compare every claim against an artifact.
- [ ] Submit early enough for upload/form recovery.
- [ ] Save submission confirmation and final digests.

## Automatic no-go conditions

Do not submit a claim of a complete live project if any of these remains:

- required Google feature is fake, decorative, local-only or silently replayed
- Gemini/ADK eligibility cannot be proved
- candidate can bypass Gateway or access oracle/reset
- unauthorized ledger release occurred without a disclosed failing result
- Python/Gemini can approve or set the gate result
- KMS attestation cannot be independently verified
- production has hardcoded auth, root/Docker socket, committed secret, or cross-tenant access
- evidence or UI obscures provenance
- a critical unresolved security vulnerability exists
- repository provenance violates clean-room rules
- hosted/video/repository access is broken

A narrower truthful submission is stronger than a broad fabricated one.
