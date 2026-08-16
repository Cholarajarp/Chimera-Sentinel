# Chimera Sentinel Implementation Phases

## Delivery strategy

The critical path is a thin, real vertical slice: candidate manifest → durable workflow → ADK case → Model Armor/Identity/Gateway → deterministic ledger → Rust decision → constrained approval → retest → KMS attestation → console and trace. Build every phase so it can be demonstrated before expanding breadth.

**Deadline:** August 31, 2026 at 5:00 PM PDT. Treat August 29 as code freeze, August 30 as submission rehearsal, and August 31 as contingency/upload day.

## Stop/go gates

| Gate | Must be green before continuing scope |
|---|---|
| G0 Foundation | clean-room repo, pinned toolchains, CI skeleton, schemas compile |
| G1 Local safety path | Rust workflow and ledger prove unauthorized release stays zero |
| G2 Managed path | required Google agent services produce live evidence with no fallback |
| G3 Certification | approval/retest and KMS attestation verify independently |
| G4 Product | durable console, trace, evaluation, deployed smoke run |
| G5 Submission | reproducible repo, video, evidence matrix, hosted URLs verified |

If a gate is red, fix it or cut later scope. Do not cover it with mock UI.

## Dated schedule

| Date | Outcome |
|---|---|
| Aug 14 | Freeze product thesis, architecture, truth rules, and clean-room disclosure. |
| Aug 15 | Create monorepo, pinned tools, schemas, CI, local dependencies, task runner. |
| Aug 16 | Rust domain, candidate/ABOM, workflow state machine, policy skeleton. |
| Aug 17 | Persistence, idempotent commands, events, authz/tenant scaffolding. |
| Aug 18 | Deterministic Rust mock ERP MCP and ledger invariant tests. **G1** |
| Aug 19 | Minimal Python ADK contract and local orchestration; corpus schema/seed cases. |
| Aug 20 | Verify/provision Registry, Runtime, identities, Gemini model and live ADK run. |
| Aug 21 | Integrate Model Armor and prove live inspect/block behavior. |
| Aug 22 | Integrate Agent Gateway/Identity and prove allowed draft + denied release. |
| Aug 23 | Integrate Memory Bank advisory retrieval; complete managed evidence adapters. **G2** |
| Aug 24 | Evidence manifest, canonical JSON vectors, Cloud Storage and KMS signing/verifier. |
| Aug 25 | Constrained approval, capability diff, retest, expiry and revocation. **G3** |
| Aug 26 | Console vertical flow, accessibility, durable event timeline and evidence views. |
| Aug 27 | OTel correlation, dashboards, privacy/redaction checks, full corpus. |
| Aug 28 | Holdout, ablations, reliability/security/E2E tests, cost and latency report. **G4** |
| Aug 29 | Deploy final revisions; security/license/secret scans; documentation code freeze. |
| Aug 30 | Record four-minute demo, execute clean-room reproduction and two rehearsals. |
| Aug 31 | Verify URLs/video/repository/Devpost fields, upload early, preserve contingency. **G5** |

Parallelize web shell/design after API contracts stabilize; infrastructure research can run beside local domain work; corpus writing can run beside adapters. Do not parallelize competing writes to shared schemas or the state machine.

## Phase 1 — Repository foundation

**Dependencies:** none.  
**Exit:** G0.

### Copy/paste coding prompt

> Initialize a new clean-room Chimera Sentinel monorepo. Read all `docs/*.md`. Do not copy any Chimera Vanguard source or assets. Create the layout from `docs/MASTER_BUILD_PROMPT.md` with a Cargo workspace, Next.js/TypeScript app using pnpm, minimal Python package using uv, Terraform root, shared JSON Schemas, task runner, CI, `.env.example`, security/contribution/notice files, and a README with prior-work disclosure. Verify current stable toolchains, choose exact compatible versions, and commit lockfiles; do not use open dependency ranges. Define shared enums for provenance and workflow state. Add format, lint, type-check, unit, schema, build, and secret-scan commands. Use synthetic identifiers only. Do not implement fake Google clients. Finish by running all foundation commands and reporting exact results.

### Acceptance criteria

- New repository history and lineage disclosure are present.
- All toolchains/dependencies are pinned and lockfiles generated.
- Rust, Python, and TypeScript “hello contract” compile and validate one shared fixture.
- CI has no hardcoded credentials and fails on format/type/test/schema error.
- Production configuration cannot enable local auth or silent adapters.
- `make bootstrap format lint check unit build` (or documented equivalents) pass.

## Phase 2 — Rust control plane

**Dependencies:** Phase 1.  
**Exit:** candidate creation and local durable workflow tests.

### Copy/paste coding prompt

> Implement the trusted Rust core without Google integrations. Create pure domain types for tenant, agent, immutable candidate revision/ABOM, policy assignment, workflow, case run, finding, approval, evidence manifest reference, attestation metadata, and revocation. Implement the exact state machine in `docs/ARCHITECTURE.md`, deterministic transition errors, versioned audit events, idempotency records, command leases, bounded retries, and optimistic concurrency. Add Axum APIs for candidate creation/read/list, certification start/read/events/cancel, findings, approval, evidence metadata, and attestation status. Use repository traits plus an in-memory test implementation and a Firestore adapter boundary; never put policy in handlers. Add OIDC-compatible auth interfaces, deny-by-default tenant authorization, RFC-style problem responses, OpenAPI, cursor pagination, request limits, and trace IDs. Implement deterministic policy truth tables returning only the four documented gate results. Add exhaustive unit/property/contract/authz/restart/duplicate-delivery tests. Run Rust format, lint with warnings denied, tests, and API schema checks.

### Acceptance criteria

- Invalid transitions and terminal mutations fail.
- Duplicate start returns the original workflow; cross-tenant access is denied.
- Restart/repository reconstruction retains canonical state in integration tests.
- Python/provider observations cannot directly set decisions.
- Every rule result includes a structured explanation and evidence references.
- Rust checks/tests pass with no ignored security-critical tests.

## Phase 3 — Deterministic MCP fixture and ledger

**Dependencies:** contracts and identity port from Phase 2.  
**Exit:** G1.

### Copy/paste coding prompt

> Implement the mock ERP as a deterministic Rust MCP-compatible service using current protocol specifications. Tools: `draft_invoice_payment`, `release_payment`, `get_payment_status`; expose reset and ledger inspection only to the test-controller/certifier oracle identity. Use strict schemas, integer minor units, synthetic references, append-only entries, idempotency keys, sequence numbers, and deterministic fixture clocks in tests. Enforce an authenticated principal and a policy adapter so the candidate cannot call privileged tools or bypass the future Gateway path. Add safe, denied-release, duplicate, reordered, malformed, overflow, unauthorized reset/inspection, replay, and concurrency tests. Implement ledger snapshots/digests and invariants, especially unauthorized released-payment delta equals zero. Integrate the invariant into Rust policy. Never allow model output to declare the ledger safe.

### Acceptance criteria

- Safe draft works; constrained candidate release is denied.
- Duplicate/reordered commands do not duplicate payment effects.
- Candidate cannot reset/read oracle or access direct deployed ingress.
- Snapshot digests reproduce and one-byte mutation is detected.
- Property/concurrency tests prove ledger accounting invariants.

## Phase 4 — ADK certifier

**Dependencies:** contracts, local candidate/fixture path.  
**Exit:** typed local case execution.

### Copy/paste coding prompt

> Verify the current official Google ADK and eligible Gemini 3.5-or-newer documentation, then implement the smallest Python ADK certifier. Pin exact dependencies. Expose authenticated internal create/status/cancel evaluation APIs. Accept only versioned immutable candidate/corpus references and return typed observations; reject unknown fields/versions, oversized requests, and arbitrary tool definitions. Implement ADK orchestration for one safe and one hostile synthetic invoice case using explicit model configuration, bounded tokens/time, trace context, and redacted structured events. Add ports for Registry, Runtime, Memory Bank, Model Armor, Gateway, and candidate invocation; local fakes must emit `LOCAL` and must never activate after live-adapter failure. Generate/validate Python models from shared schemas. Test malformed model responses, timeout, cancellation, redaction canaries, contract compatibility, and inability to approve/sign/set final status.

### Acceptance criteria

- Rust/Python golden contract fixtures round-trip.
- Local safe/hostile runs return complete `LOCAL` evidence.
- Raw fixture text and canary secret do not appear in logs/traces/errors.
- Provider failure is explicit and never falls back.
- Python lint, type, contract, and tests pass.

## Phase 5 — Google managed integrations

**Dependencies:** Phases 2–4 and cloud project access.  
**Exit:** G2.

### Copy/paste coding prompt

> Implement live Google Cloud adapters required by the Fortified Enterprise Fleet track. Before coding each adapter, verify official product availability, region, API, IAM, quotas, service identity, and current resource/model names; record an ADR and pin compatible SDKs. Provision Agent Registry, long-running Agent Runtime, separate Agent Identities, Agent Gateway policy, Model Armor template/configuration, Memory Bank, Gemini 3.5+, and async transport using Terraform where supported and idempotent reviewed setup scripts otherwise. Bind the candidate revision to registry/runtime/identity/configuration resources. Prove: live ADK/Gemini execution; direct injection Model Armor disposition; candidate identity allowed to draft; same identity denied `release_payment` through Gateway; no direct ERP bypass; analyst-approved memory retrieval; asynchronous run lifecycle. Persist sanitized provider references, timestamps, config hashes, trace IDs, and `LIVE` provenance. Add positive/negative IAM, service-error, timeout, stale-reference, and mode-isolation tests. Never fake unsupported resources or use badges as proof.

### Acceptance criteria

- Every required managed feature has a verified resource and live evidence artifact.
- Candidate and certifier identities are distinct and least privileged.
- Gateway live test permits draft and denies release; ledger remains unchanged.
- Model Armor live block/allow evidence correlates to cases.
- Memory ablation confirms no authorization effect.
- Runtime is genuinely asynchronous and survives client disconnect.
- Required-service outage blocks certification without local fallback.

## Phase 6 — Evidence and attestation

**Dependencies:** durable workflow and live evidence.  
**Exit:** independent cryptographic verification.

### Copy/paste coding prompt

> Implement evidence finalization and attestation in Rust. Store immutable case/provider/ledger/approval objects in Cloud Storage and metadata in Firestore. Produce a manifest containing schema, media type, size, producer, provenance, timestamp, and SHA-256 for every object. Use RFC 8785/JCS or a documented test-vectored canonicalization library; verify object digests after write before committing the manifest. Construct the exact versioned payload in `docs/ARCHITECTURE.md`, including registry/runtime/identity/source/model/prompt/tool/memory/Gateway/Model Armor/policy/corpus/evaluation/evidence/capability/approval/time bindings. Use Cloud KMS workload identity and an explicit asymmetric signing algorithm. Persist key version and signature only after local verification. Add `sentinelctl attestation verify` with revocation/expiry/audience/environment checks. Test golden vectors, one-byte tamper, omitted object, wrong revision/key/environment, not-before, expiry, revocation, KMS error, duplicate signing, and interrupted finalization.

### Acceptance criteria

- Independent CLI verifies a valid exported envelope without UI trust.
- All tamper/stale/wrong-scope tests reject.
- KMS failure cannot produce `Certified`.
- Duplicate finalization creates one terminal attestation.
- Manifest is complete, content-addressed, and reproducible.

## Phase 7 — Approval, least privilege, and retest

**Dependencies:** policy, live path, evidence.  
**Exit:** G3.

### Copy/paste coding prompt

> Complete the flagship constrained-approval workflow. When safe drafting works but payment release is excessive or unsafe, Rust must return `CONSTRAINED_APPROVAL_REQUIRED` with an explainable capability diff. Implement authorized approval/rejection with reason, scope, actor, policy version, issue/expiry times, optimistic concurrency, separation-of-duty checks, and immutable audit event. Never let approval override a ledger invariant, missing identity, invalid signature, unknown revision, or missing managed evidence. Apply only a permission reduction, then automatically dispatch a positive draft smoke case and negative release case. Certify only after both pass and evidence finalizes. Add manual revocation and attestation expiry. Test self/cross-tenant/expired/stale/double approval, capability broadening, failed retest, and race conditions.

### Acceptance criteria

- Dangerous capability can only be removed, not broadened.
- Authorized review is attributable and expiring.
- Safe draft remains functional after restriction; release remains denied.
- Failed retest blocks signing.
- Revoked/expired attestation fails verifier and UI authorization.

## Phase 8 — Enterprise console

**Dependencies:** stable APIs and workflow.  
**Exit:** judge-ready durable journey.

### Copy/paste coding prompt

> Build the Next.js enterprise console against generated/schema-checked APIs. Implement fleet posture; agent/revision ABOM; start certification; resumable timeline; case matrix; findings/evidence; model/tool/policy/capability diff; constrained approval/rejection; attestation details, independent verification status, expiry/revocation; and correlated Cloud trace link. All data must display accessible provenance labels. Render explicit loading, empty, error, stale, disconnected, unauthorized, blocked, and partial states. Stream events with secure cookie/header authorization, sequence validation, reconnect and API backfill—never URL tokens. Do not store provider keys or canonical reports in the browser. Add role-aware controls, keyboard support, focus management, contrast, narrow-screen layout, bundle review, component tests, accessibility tests, and Playwright flows for blocked and certified paths.

### Acceptance criteria

- Reload and a second browser session show identical server state.
- Cosmetic controls cannot imply success before API confirmation.
- Provenance, permission, expiry, and error states are unambiguous.
- Critical journey works at desktop and narrow viewport with keyboard.
- Type/lint/unit/accessibility/Playwright/production build pass.

## Phase 9 — OpenTelemetry and operations

**Dependencies:** all services.  
**Exit:** one safe correlated trace and usable health signals.

### Copy/paste coding prompt

> Add OpenTelemetry end to end using W3C trace context across web request, Rust API/worker, async command, Python ADK, candidate/Gateway/MCP evidence, storage, policy, and KMS where APIs permit. Define an allowlist of safe attributes and separate append-only audit events from logs. Add redaction processors and canary tests proving prompts, invoices, tool payloads, auth headers, tokens, secrets, PII, and chain-of-thought do not export. Emit metrics for workflow outcome/duration, queue age, retries, Model Armor/Gateway outcomes, ledger violations, evidence/signing failures, token/cost, and attestation expiry. Provision Cloud dashboards/alerts and document trace navigation. Test exporter failure and ensure it cannot falsely approve a workflow.

### Acceptance criteria

- Live trace correlates the demo path and contains no protected payload.
- Required metrics appear with bounded cardinality.
- Canary secret search returns no telemetry match.
- Audit records remain durable if debug telemetry is sampled.
- Export failure behavior matches policy and is visible.

## Phase 10 — Evaluation and hardening

**Dependencies:** full vertical path.  
**Exit:** G4.

### Copy/paste coding prompt

> Implement and run `docs/EVALUATION_AND_DEMO.md`. Create exactly 80 versioned synthetic cases with the documented categories/counts, 64 development and 16 sealed holdout cases, deterministic fixture state, expected control outcome, tool constraints, ledger invariants, and provenance. Validate corpus uniqueness and leakage. Run full system, baseline, and one-control-at-a-time ablations; repeat nondeterministic conditions as specified. Calculate confusion matrices, unauthorized-side-effect escape rate, safe-task completion, evidence completeness, reliability, p50/p95 latency, tokens, cost, drift/replay agreement, and confidence intervals where valid. Run restart, duplicate, out-of-order, throttling, cross-tenant, bypass, tamper, expiry, revocation, redaction, dependency, container, IaC, and E2E tests. Produce machine-readable results and a concise limitations report. Do not tune on holdout failures or hide infrastructure failures.

### Acceptance criteria

- Corpus count/split/categories match manifest and schema.
- Holdout remained sealed until one declared final run.
- Ledger oracle shows zero unauthorized releases in claimed passing run.
- All metrics include denominator, environment, versions, timestamp, and failures.
- Ablations demonstrate defense-in-depth without implying universal detection.
- Critical/high findings are fixed or explicitly block release.

## Phase 11 — Deployment and submission

**Dependencies:** G4.  
**Exit:** G5 before deadline.

### Copy/paste coding prompt

> Prepare a reproducible hosted submission. Pin and scan production images, deploy by immutable revision/digest where supported, apply least-privilege IAM, budgets/quotas/scaling bounds, synthetic seed data, and verified health checks. Run a clean-environment deployment from README commands and the full live flagship smoke test. Generate deployment evidence containing sanitized project/region/resource/revision identifiers and trace/evidence links. Complete `docs/HACKATHON_CHECKLIST.md`, architecture diagrams, limitations, clean-room disclosure, setup/teardown, security model, evaluation report, and license attribution. Record an approximately four-minute English demo using the exact script, showing live Google Cloud proof and independent signature verification. Verify hosted URL, repository accessibility, video permissions/audio/captions, Devpost copy, and all links from a logged-out session. Freeze code Aug 29; use Aug 31 only for verified blocker fixes and early upload.

### Acceptance criteria

- A clean operator follows README to deploy/smoke without undocumented steps.
- Hosted console and APIs use no placeholder URL or demo auth bypass.
- Submission matrix has implementation proof, demo proof, artifact, owner/status, and fail condition for every requirement.
- Video is near four minutes, legible, captioned, and shows live resources/trace.
- No secret, unsupported claim, fake integration, broken link, critical vulnerability, or unlicensed asset remains.

## Risk register and response

| Risk | Early signal | Response |
|---|---|---|
| Managed service/API changes | docs/API mismatch | Verify Aug 15–20, isolate adapters, preserve exact evidence; escalate blocker immediately |
| Agent service regional incompatibility | provisioning failure | Select compatible region before stable infra; document architecture constraint |
| Too much scope | G1/G2 late | Cut roadmap/UI breadth; protect flagship path |
| Python expands into control plane | decision/state logic appears there | Move logic to Rust and add boundary test |
| Demo network/provider failure | flaky rehearsal | Keep immutable `REPLAY` backup clearly labeled; never call it live |
| Corpus overfits | holdout inspected early | Seal hash/access; one declared final run |
| Sensitive data leakage | telemetry canary found | block release; remove source, rotate affected secrets, rerun |
| IAM too broad | owner/editor/basic roles | replace with custom/minimal roles and negative tests |
| KMS/signature mismatch | verifier disagreement | lock canonicalization and algorithm with golden vectors early |
| Async duplication | repeated side effect | idempotency at command, case, tool and signing layers |
| Submission upload delay | late video/render | freeze Aug 29, upload draft early, validate logged-out access |

## Future-ready extension order

After submission, prioritize production foundations before speculative intelligence: tenant isolation and SSO → policy packs and CI admission → revocation/drift → MCP supply-chain inventory → incident response → enterprise connectors → HA/DR and independent security review. Keep adaptive attacks, graph analytics, and automated permission synthesis in an explicitly evaluated R&D track.
