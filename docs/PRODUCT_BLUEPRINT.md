# Chimera Sentinel Product Blueprint

## Product thesis

Chimera Sentinel is an **admission control and continuous assurance platform for enterprise AI agents**. Before an agent revision receives production permissions, Sentinel evaluates the exact revision, identity, model, prompts, tools, defenses, and policy; records tamper-evident evidence; requires approval where needed; and issues an identity-bound, expiring attestation.

It is not another chatbot scanner. It is the control point between **an agent build** and **production authority**.

> Flagship promise: no agent revision is promoted unless the evidence required by policy exists, deterministic safety invariants pass, and any required human approval is recorded.

This is an operational guarantee of Sentinel's gate behavior—not a claim that an agent is perfectly safe, formally verified, or free of unknown vulnerabilities.

## Why enterprises need it

Agent deployments combine mutable models, prompts, tools, machine identities, memory, data, and runtime policies. Existing AppSec tools inspect source or packages but rarely answer:

- Which exact agent revision has authority right now?
- Which identity and tools were tested together?
- Did a realistic hostile task cause a forbidden business-side effect?
- Was an exception approved, by whom, for how long, and against which evidence?
- Can authorization be revoked when the model, policy, tool, or runtime drifts?

Sentinel answers these questions with a durable release workflow and verifiable evidence.

## Positioning

**Category:** Agent security posture management plus release admission control.

**Primary buyer:** CISO, AI platform leader, or application security leader.

**Primary users:** AI platform engineers, agent developers, security reviewers, risk/audit teams, and incident responders.

**Hackathon wedge:** Certify a procurement/accounts-payable agent against prompt-injection and unauthorized-payment risks on Google Cloud.

**Long-term platform:** Govern build-time admission, runtime posture, drift, exceptions, revocation, and evidence across an enterprise agent fleet.

## Product-truth contract

1. Every displayed datum is labeled `LIVE`, `REPLAY`, `DEMO`, `INFERRED`, or `LOCAL`.
2. Gemini recommends, classifies, and summarizes. Rust performs deterministic policy evaluation and final gate decisions.
3. Memory Bank is advisory context. It never grants permissions, approves exceptions, or changes canonical workflow state.
4. Missing evidence, expired evidence, scanner failure, timeout, schema mismatch, and infrastructure error block promotion.
5. An attestation means the named revision passed the named tests under the named configuration. It is not a universal safety certificate.
6. No raw prompts, credentials, invoice content, secret values, or unnecessary PII enter telemetry.
7. A managed-service badge is shown only when a live call and correlated evidence prove the integration.
8. Local and replay modes never silently fall back from failed cloud integrations.

## Personas and journeys

### AI platform engineer

Registers an agent candidate, selects a policy pack, starts certification, observes progress, resolves infrastructure failures, and promotes only after a valid attestation exists.

**Success:** can reproduce a decision from immutable inputs and can verify the signature without trusting the UI.

### Agent developer

Receives actionable findings tied to a concrete case, tool call, policy rule, and evidence object; submits a corrected revision; compares results without exposing sensitive payloads.

**Success:** understands why admission failed and can retest without security granting broad production access.

### Security reviewer

Reviews proposed capability reduction, evidence completeness, residual risks, and prior analyst-approved dispositions; approves or rejects a time-bounded exception.

**Success:** every decision is attributable, scoped, expiring, and revocable.

### Auditor or risk officer

Queries who approved which agent identity and policy, verifies signed attestations, exports evidence manifests, and checks retention without accessing raw business data.

**Success:** obtains a trustworthy audit trail, not screenshots or browser-local reports.

### Incident responder

Sees drift or suspicious tool denials, revokes an attestation, quarantines a revision, and links the incident to the last known-good certification.

**Success:** containment is fast, recorded, and does not depend on editing application code.

## Flagship end-to-end workflow

1. A new invoice-agent revision is registered in quarantine with source digest, runtime resource, machine identity, model, prompt/config hashes, declared tools, and requested permissions.
2. Rust validates the candidate manifest and creates an idempotent durable certification workflow.
3. A Python Google ADK certifier executes safe, hostile, and mixed invoice cases asynchronously.
4. Model Armor blocks a direct prompt injection; the evidence records the disposition without retaining sensitive content.
5. An obfuscated instruction may reach the candidate, but Agent Gateway denies `release_payment` because the candidate identity lacks that capability.
6. A deterministic mock ERP ledger proves that no payment was released. Model text cannot override this oracle.
7. Memory Bank supplies a previously analyst-approved disposition as advisory context, visibly labeled `LIVE` or `REPLAY`.
8. Rust validates evidence completeness and computes `CONSTRAINED_APPROVAL_REQUIRED` because safe invoice drafting works but payment release is unsafe or unnecessary.
9. A human reviewer approves a least-privilege policy removing `release_payment`; approval has scope, reason, identity, timestamp, and expiry.
10. Sentinel reruns smoke and negative tests. Valid drafting succeeds while direct payment release remains denied.
11. Cloud KMS signs an attestation binding the exact registry resource, agent identity, runtime, source digest, prompt/config digest, model reference, tool manifest, gateway policy hash, Model Armor configuration hash, evaluation digest, decision, issue time, and expiry.
12. The console verifies the signature and shows a correlated OpenTelemetry trace in Google Cloud.

## True hackathon MVP

| Capability | Acceptance criteria |
|---|---|
| Candidate registry view | A candidate can be created and retrieved by stable ID; its immutable revision digest and Google registry resource are visible; mutation creates a new revision. |
| Durable certification | Restarting a worker does not lose state; duplicate commands do not duplicate cases or attestations; terminal decisions are immutable. |
| Typed ADK boundary | Python accepts a versioned request and returns schema-valid case evidence; it cannot approve, sign, or set the final decision. |
| Adversarial evaluation | Safe and hostile fixtures execute; every case has expected and observed outcomes; skipped or malformed cases block admission. |
| Model Armor evidence | At least one live test proves inspection/block behavior and records a provider evidence reference. If unavailable, the certification fails rather than simulating success. |
| Identity-scoped Gateway | A live tool invocation identifies the caller and denies unauthorized `release_payment`; direct backend access is not exposed to the candidate. |
| Deterministic side-effect oracle | Ledger invariants prove no unauthorized payment was created, released, or mutated, independent of model output. |
| Advisory memory | An approved disposition can inform analysis, but changing memory cannot change authorization or the final deterministic rule result. |
| Human constrained approval | Only an authorized reviewer can approve; the approval is capability-scoped, justified, expiring, auditable, and followed by retest. |
| Evidence bundle | A content-addressed manifest links cases, sanitized events, policies, approvals, hashes, and cloud references; missing objects invalidate it. |
| Signed attestation | KMS signs a canonical payload; an independent verification command detects payload or signature tampering and rejects expired/revoked attestations. |
| Enterprise console | Fleet, candidate, run, finding, approval, evidence, and attestation views use durable APIs and display provenance labels. |
| Observability | One trace correlates API request, workflow, ADK run, gateway denial, evidence finalization, and signing without sensitive payloads. |
| Reproducible deployment | A clean environment can deploy documented components with pinned dependencies and no committed secrets or undocumented manual steps. |

## Enterprise capability map

These capabilities prevent Sentinel from becoming a one-demo scanner. Only the items marked **MVP** are hackathon commitments.

### 1. Fleet inventory and Agent Bill of Materials

An **Agent Bill of Materials (ABOM)** records model/version, prompt and configuration digests, tools/MCP servers, machine identity, memory stores, data classifications, runtime, owners, dependencies, policies, and deployment environments.

- **MVP:** immutable candidate manifest and revision comparison.
- **Next:** discovery/import from registries and CI systems; ownership and criticality; stale/unowned-agent alerts.
- **Acceptance:** a production authorization always resolves to one ABOM revision; unknown or mutable dependencies block promotion.

### 2. Policy-as-code and control packs

Versioned policy packs map business risk to required tests, evidence, separation of duties, capabilities, expiry, and deployment conditions.

- **MVP:** deterministic Rust rules for the AP scenario.
- **Next:** signed organization packs, inheritance, dry-run simulation, explainable rule traces, and controls mapped to OWASP GenAI, NIST AI RMF, and internal requirements.
- **Future option:** adopt a standard policy language only after equivalence, sandboxing, and deterministic evaluation are proven.
- **Acceptance:** identical inputs and policy version produce identical decisions; every denial identifies its rule and missing/failed evidence.

### 3. Identity and least privilege

Bind certification to workload identity, tools, resources, actions, environment, and time.

- **MVP:** separate certifier and candidate identities; gateway denies an unauthorized payment action.
- **Next:** permission-diff recommendations, just-in-time grants, break-glass workflow, identity rotation, and dormant privilege detection.
- **Acceptance:** replacing the identity or broadening a tool permission invalidates or revokes the attestation.

### 4. Tool and MCP supply-chain security

Maintain tool manifests, schemas, publisher/source, package/container digests, network destinations, privilege, and data classification.

- **MVP:** typed mock ERP MCP tools with strict schemas and immutable ledger.
- **Next:** MCP server allowlists, signature/provenance checks, schema drift alarms, dependency/SBOM ingestion, egress policies, and tool-behavior canaries.
- **Acceptance:** an undeclared tool, changed schema, mutable image tag, or unapproved destination blocks admission.

### 5. Evaluation and attack simulation

Versioned safe, hostile, mixed, multilingual, encoding, tool-response, memory-poisoning, and workflow-confusion cases with deterministic business invariants.

- **MVP:** approximately 80 cases and holdout evaluation.
- **Next:** tenant-owned corpora, attack mutation, regression budgets, shadow evaluation, and independent red-team review.
- **R&D:** adaptive adversarial generation; never report generated cases as independent proof without a held-out oracle.
- **Acceptance:** results are reproducible, corpus versions are recorded, and model judgments never replace side-effect invariants.

### 6. Evidence, attestations, and promotion enforcement

Generate portable, verifiable, expiring release evidence.

- **MVP:** content-addressed bundle and KMS signature.
- **Next:** CI/CD admission API, deployment-controller verification, revocation list, key rotation, transparency log, and offline verifier.
- **Acceptance:** stale, tampered, revoked, wrong-environment, or wrong-revision attestations cannot authorize promotion.

### 7. Runtime posture, drift, and revocation

Certification is not permanent. Monitor configuration, model alias, prompts, tools, identity, gateway policy, defense settings, and behavior signals.

- **MVP:** attestation expiry and manual revocation state.
- **Next:** event-driven drift detection, scheduled recertification, canary transactions, automatic quarantine policy, and last-known-good rollback recommendation.
- **Acceptance:** critical drift moves authorization to `SUSPENDED` or `RECERTIFICATION_REQUIRED`; automation cannot silently expand privilege.

### 8. Incident response and forensics

Connect runtime events to certified state without collecting sensitive content by default.

- **Next:** incident timelines, evidence preservation, scoped quarantine/revocation, case management, export to SIEM/SOAR, and post-incident recertification.
- **Acceptance:** responders can answer which revision, identity, policy, tool, approval, and attestation were active at an event time.

### 9. Exception and risk acceptance lifecycle

Treat exceptions as expiring risk objects, not permanent bypass flags.

- **MVP:** constrained approval with expiry.
- **Next:** dual control, compensating controls, renewal workflow, exception budgets, owner notifications, and automatic expiry.
- **Acceptance:** no exception can suppress missing identity, invalid signature, unknown revision, or failed ledger invariant.

### 10. Multi-tenant enterprise administration

- **Next:** tenant isolation, environments, projects, RBAC/ABAC, SSO/OIDC, SCIM, service accounts, separation of duties, delegated administration, quotas, retention, legal hold, regional storage, customer-managed keys, and audit export.
- **Acceptance:** cross-tenant identifiers cannot retrieve data; every administrative mutation is authorized and audit logged; destructive retention actions are reviewable.

### 11. Integrations and developer platform

- **Next:** stable API/SDK, webhooks with signature and retries, CI checks, Terraform provider/resources, GitHub/GitLab/Cloud Build integration, ticketing, SIEM, registry connectors, and policy-pack CLI.
- **Acceptance:** integrations are idempotent, versioned, observable, least-privileged, and testable in a sandbox.

### 12. Reliability and operations

- **MVP:** retry-safe workflows, bounded concurrency, explicit failure states, health/readiness checks, and cost/latency telemetry.
- **Next:** regional failover, backup/restore drills, RPO/RTO objectives, queue dead-letter handling, load tests, capacity limits, SLOs, runbooks, and support diagnostics.
- **Acceptance:** infrastructure failure never creates an approval; recovery does not duplicate side effects or signatures.

### 13. Privacy and data governance

- **MVP:** redacted telemetry, synthetic demo invoices, configurable evidence retention.
- **Next:** field-level classification, tenant retention, deletion workflow, regionalization, DLP integration, consent/legal basis metadata, and privacy-preserving fixture generation.
- **Acceptance:** secrets and raw protected business documents are not required for routine observability or audit verification.

### 14. Compliance evidence, not compliance theater

Sentinel can map evidence to controls and export audit artifacts. It must not claim that using Sentinel makes an organization compliant.

- **Next:** control mappings, evidence schedules, reviewer sign-off, auditor read-only portal, and chain-of-custody export.
- **Acceptance:** mappings identify source evidence, collection time, scope, owner, and limitations.

## Non-goals for the hackathon

- General-purpose SOC/SIEM or endpoint protection
- Autonomous production remediation
- Multi-cloud parity
- GKE-first deployment
- Custom vector database
- Full compliance certification
- Formal proof of model safety
- Replacement for SAST, SCA, DLP, IAM, or a gateway
- All speculative Vanguard roadmap modules

## Prioritized roadmap

### Stage 0 — Hackathon proof

Deliver one live, polished admission workflow, signed evidence, console, managed Google integrations, evaluation, and reproducible deployment. Cut everything else before weakening this path.

### Stage 1 — Design partner pilot (0–3 months)

ABOM inventory, policy pack v1, CI admission check, revocation, scheduled recertification, OIDC SSO, basic RBAC, audit export, tenant boundaries, retention, webhook API, backup/restore, and one non-AP fixture.

### Stage 2 — Enterprise beta (3–9 months)

Registry discovery, tool/MCP inventory, drift engine, incident timeline, dual-control exceptions, SCIM, customer-managed keys, signed policy packs, deployment verifier, SIEM/ticketing connectors, SLOs, load/security testing, and regional options.

### Stage 3 — General availability (9–18 months)

HA/DR commitments, independent security assessment, granular ABAC, evidence/legal hold, transparency/revocation services, SDKs, supported connectors, policy marketplace governance, billing/quotas, and support runbooks.

### Research track (never marketed as committed capability)

Behavioral anomaly detection, attack mutation, causal traces, privacy-preserving cross-tenant intelligence, automated permission synthesis, and graph-based blast-radius analysis. Each requires independent evaluation, abuse controls, false-positive analysis, and a deterministic enforcement boundary.

## Competitive differentiation

| Common approach | Sentinel difference |
|---|---|
| Chatbot red-team report | Release gate bound to exact deployable inputs and identity |
| Static source scan | Runtime tool, identity, defense, and business-side-effect evidence |
| Gateway alone | Gateway policy plus certification, human governance, and signed provenance |
| LLM judge | Deterministic rules and ledger oracle; LLM output is evidence, not authority |
| One-time dashboard | Expiry, drift, recertification, and revocation lifecycle |
| Compliance screenshots | Content-addressed evidence and independently verifiable signatures |

## Measurable meaning of “100× better”

“100×” is a product target, not a current factual claim. It must be demonstrated with explicit baselines:

1. **100× faster evidence retrieval:** target under 30 seconds to locate a release's identity, policy, approval, and test evidence versus a measured manual baseline of over 50 minutes.
2. **100× smaller unauthorized-action window:** target revocation/quarantine under one minute versus a measured manual process over 100 minutes.
3. **100× more reproducible decisions:** target 99%+ deterministic replay agreement versus a measured ad hoc review baseline below 1%; compare only equivalent decision sets.
4. **Zero ambiguous promoted revisions:** 100% of Sentinel-authorized promotions must bind immutable revision and policy digests; this is a gate invariant, not a claim about all enterprise deployments.
5. **No unauthorized ledger side effects in the evaluated corpus:** report numerator, denominator, confidence interval, corpus version, and limitations.

Do not publish a multiplier until the baseline, sample, method, and result are reproducible.

## Product success metrics

- Security: unauthorized side-effect escape rate, critical invariant pass rate, gateway denial coverage, stale-attestation rejection rate
- Governance: percent of production agents with complete ABOM, valid attestation, owner, and unexpired policy
- Operations: certification completion rate, p50/p95 duration, queue age, retry recovery, revocation latency
- Developer experience: remediation-to-retest time, false-positive dispute rate, actionable finding rate
- Evidence: independent verification success, missing-object rate, audit retrieval time
- Cost: cost per certification and per case, model-token budget variance

## Definition of product success

For the hackathon, success is a judge seeing a real Google Cloud workflow stop a dangerous action, preserve useful behavior, require accountable human approval, and produce independently verifiable evidence. For the company, success is becoming the trusted authorization layer through which enterprise agent revisions earn, retain, and lose production authority.
