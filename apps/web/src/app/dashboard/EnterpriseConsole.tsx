'use client';

import React, { useState, useMemo, useEffect, useCallback, useRef } from 'react';
import Image from 'next/image';
import {
  Activity,
  ArrowRight,
  CheckCircle2,
  Cloud,
  Database,
  Download,
  Fingerprint,
  LockKeyhole,
  Play,
  ShieldCheck,
  X,
} from 'lucide-react';

import { motion, AnimatePresence } from 'framer-motion';
import { cn } from '@/lib/utils';
import { SidebarNav, TabType, TABS } from '@/components/dashboard/SidebarNav';
import { DashboardProvider } from '@/components/dashboard/DashboardContext';
import { FleetView } from '@/components/dashboard/views/FleetView';
import { CandidateView } from '@/components/dashboard/views/CandidateView';
import { CertificationView } from '@/components/dashboard/views/CertificationView';
import { CorpusView } from '@/components/dashboard/views/CorpusView';
import { ApprovalView } from '@/components/dashboard/views/ApprovalView';
import { EvidenceView } from '@/components/dashboard/views/EvidenceView';
import { AttestationView } from '@/components/dashboard/views/AttestationView';
import { TraceView } from '@/components/dashboard/views/TraceView';
import { PolicyView } from '@/components/dashboard/views/PolicyView';
import { VulnerabilityView } from '@/components/dashboard/views/VulnerabilityView';


const API_BASE = '/api';

const TENANT_ID = '00000000-0000-0000-0000-000000000001';

const CORPUS_PAGE_SIZE = 20;

const MAX_POLL_ATTEMPTS = 90;



interface Vulnerability {
  id: string;
  title: string;
  severity: string;
  description: string;
}

type Provenance = 'LIVE' | 'REPLAY' | 'SYSTEM_TEST' | 'INFERRED' | 'LOCAL';

interface ScanResponse {
  status: string;
  provenance: Provenance;
  vulnerabilities: Vulnerability[];
}

type WorkflowStep =
  | 'Registered'
  | 'Queued'
  | 'Running'
  | 'EvidencePending'
  | 'ApprovalRequired'
  | 'RetestRequired'
  | 'Attesting'
  | 'Certified'
  | 'Failed'
  | 'Blocked'
  | 'Rejected'
  | 'Cancelled'
  | 'Revoked'
  | 'RecertificationRequired';

type ExperienceMode = 'replay' | 'live';



/** Assertions a corpus case makes about required candidate behaviour. */
interface CaseExpectations {
  expected_outcome: string;
  expected_model_armor_disposition: string;
  expected_gateway_disposition: string;
  allowed_tools: string[];
  forbidden_tools: string[];
  requested_tool: string | null;
  unauthorized_released_payment_delta: number;
}

/**
 * A case *definition* served by the trusted core from corpus/v1.
 * Definitions never carry observed results; those are run evidence.
 */
interface TestCase {
  case_id: string;
  category: string;
  category_name: string;
  split: string;
  title: string;
  description: string;
  sealed: boolean;
  fixture: Record<string, unknown> | null;
  expectations: CaseExpectations;
  source_path: string;
  sha256: string;
  digest_verified: boolean;
}

interface CorpusIntegrity {
  manifest_sha256: string;
  declared_cases: number;
  loaded_cases: number;
  digest_verified_cases: number;
  all_digests_verified: boolean;
}

interface CorpusResponse {
  schema_version: string;
  corpus_version: string;
  name: string;
  description: string;
  total_cases: number;
  development_cases: number;
  holdout_cases: number;
  categories: Array<{
    id: string;
    name: string;
    dev_count: number;
    holdout_count: number;
    total: number;
  }>;
  integrity: CorpusIntegrity;
  cases: TestCase[];
  provenance: string;
}

/** Immutable Agent Bill of Materials as recorded by the control plane. */
interface AgentBillOfMaterials {
  source_digest: string;
  registry_resource: string;
  runtime_resource: string;
  agent_identity: string;
  model_ref: string;
  prompt_config_digest: string;
  tool_manifest_digest: string;
  memory_config_digest: string;
  gateway_policy_digest: string;
  model_armor_config_digest: string;
  requested_capabilities: string[];
  data_classification: string[];
  environment: string;
  owner: string;
  risk_tier: string;
  provenance: string;
  recorded_at: string;
}

interface CandidateDetail {
  tenant_id: string;
  agent_id: string;
  revision_id: string;
  abom: AgentBillOfMaterials;
  policy_pack_id: string;
  corpus_version: string;
  created_at: string;
}

interface CandidateListResponse {
  candidates: CandidateDetail[];
  next_cursor: string | null;
}

/** Append-only audit event recorded by the trusted core. */
interface AuditEvent {
  event_id: string;
  workflow_id: string | null;
  event_type: string;
  actor: string;
  before_state: string | null;
  after_state: string | null;
  decision: string | null;
  explanation: string | null;
  provenance: string;
  timestamp: string;
  trace_id: string | null;
}

interface AuditListResponse {
  events: AuditEvent[];
}

interface WorkflowApiResponse {
  state: string;
}

interface WorkflowSummary {
  workflow_id: string;
  candidate_revision_id: string;
  state: string;
  gate_decision: string | null;
  updated_at: string;
}

interface WorkflowListResponse {
  workflows: WorkflowSummary[];
}

interface FleetPostureResponse {
  total_workflows: number;
  certified: number;
  blocked: number;
  approval_required: number;
  running: number;
  posture: 'HEALTHY' | 'ATTENTION_REQUIRED';
  provenance: 'LIVE';
}

interface EvidenceObjectRef {
  path: string;
  media_type: string;
  schema_version: string;
  sha256_digest: string;
  size_bytes: number;
  producer: string;
  provenance: string;
}

interface EvidenceManifestResponse {
  manifest: {
    manifest_digest: string;
    objects: EvidenceObjectRef[];
  } | null;
}

interface AttestationResponse {
  attestation: Record<string, unknown> | null;
}

interface AttestationVerifyResponse {
  valid: boolean;
  details: string;
}

const API_WORKFLOW_STATES: Record<string, WorkflowStep> = {
  REGISTERED: 'Registered',
  QUEUED: 'Queued',
  RUNNING: 'Running',
  EVIDENCE_PENDING: 'EvidencePending',
  APPROVAL_REQUIRED: 'ApprovalRequired',
  RETEST_REQUIRED: 'RetestRequired',
  ATTESTING: 'Attesting',
  CERTIFIED: 'Certified',
  FAILED: 'Failed',
  BLOCKED: 'Blocked',
  REJECTED: 'Rejected',
  CANCELLED: 'Cancelled',
  REVOKED: 'Revoked',
  RECERTIFICATION_REQUIRED: 'RecertificationRequired',
};

function parseWorkflowState(state: string): WorkflowStep | null {
  return API_WORKFLOW_STATES[state] ?? null;
}

/** Abbreviates a `sha256:<hex>` digest for table display. */
function shortDigest(digest: string): string {
  const hex = digest.startsWith('sha256:') ? digest.slice(7) : digest;
  return hex.length <= 16 ? digest : `sha256:${hex.slice(0, 8)}…${hex.slice(-4)}`;
}

function riskClass(tier: string): string {
  const normalized = tier.toUpperCase();
  if (normalized === 'HIGH' || normalized === 'CRITICAL') return 'is-high';
  if (normalized === 'MEDIUM') return 'is-medium';
  return 'is-low';
}

function auditSeverity(eventType: string): string {
  if (/FAILED|REJECTED|CANCELLED|REVOKED|DRIFT/.test(eventType)) return 'is-alert';
  if (/APPROVAL|ATTESTATION/.test(eventType)) return 'is-decision';
  return 'is-normal';
}

/** Capabilities that move money or destroy data require reduction before release. */
function isElevatedCapability(capability: string): boolean {
  return /release|delete|transfer|approve_payment/i.test(capability);
}

/**
 * Checks the control-plane verifier applies to an attestation.
 *
 * The verifier returns one aggregate verdict rather than per-check results, so
 * this list documents the policy. Individual rows are never marked passing on
 * their own.
 */
const VERIFICATION_CHECKS: ReadonlyArray<{ id: number; label: string; detail: string }> = [
  { id: 1, label: 'Cryptographic KMS signature', detail: 'RSA-PSS signature over the RFC 8785 canonical payload' },
  { id: 2, label: 'Key reference match', detail: 'Signing key version matches the expected KMS resource' },
  { id: 3, label: 'Validity window', detail: 'Attestation has not expired' },
  { id: 4, label: 'Revocation status', detail: 'Attestation has not been revoked' },
  { id: 5, label: 'Tenant audience match', detail: 'Attestation was issued for this tenant' },
  { id: 6, label: 'Target environment match', detail: 'Attestation authorizes the requested environment' },
  { id: 7, label: 'Candidate revision match', detail: 'Attestation binds the exact deployable revision' },
  { id: 8, label: 'Evidence manifest digest', detail: 'Referenced evidence manifest digest resolves' },
];

export default function EnterpriseConsole() {
  const [activeTab, setActiveTab] = useState<TabType>('fleet');
  const [experienceMode, setExperienceMode] = useState<ExperienceMode>('live');
  const [workflowState, setWorkflowState] = useState<WorkflowStep>('Registered');
  const [isRunningSim, setIsRunningSim] = useState(false);
  const [isTampered, setIsTampered] = useState(false);
  const [selectedCase, setSelectedCase] = useState<TestCase | null>(null);
  const caseDialogRef = useRef<HTMLDivElement>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [splitFilter, setSplitFilter] = useState<'all' | 'development' | 'holdout'>('all');
  const [categoryFilter, setCategoryFilter] = useState<string>('all');
  const [corpusPage, setCorpusPage] = useState(0);
  // Corpus definitions come from the trusted core; never reconstructed locally.
  const [corpus, setCorpus] = useState<CorpusResponse | null>(null);
  const [corpusStatus, setCorpusStatus] = useState<'loading' | 'ready' | 'error'>('loading');
  const [corpusError, setCorpusError] = useState<string | null>(null);
  const [reviewerRole, setReviewerRole] = useState('sec_reviewer');
  const [approvalGranted, setApprovalGranted] = useState(false);
  const [expiryDays, setExpiryDays] = useState(90);
  // Set at sign-in; the control plane compares this against the candidate owner.
  const [reviewerPrincipal, setReviewerPrincipal] = useState<string | null>(null);
  // Candidate ABOMs and audit events are tenant runtime state, so they exist
  // only when the control plane is reachable. Never substituted with samples.
  const [candidates, setCandidates] = useState<CandidateDetail[]>([]);
  const [auditEvents, setAuditEvents] = useState<AuditEvent[]>([]);
  const [selectedRevisionId, setSelectedRevisionId] = useState<string | null>(null);
  // Live API state
  const [liveWorkflowId, setLiveWorkflowId] = useState<string | null>(null);
  const [apiError, setApiError] = useState<string | null>(null);
  const [fleetPosture, setFleetPosture] = useState<FleetPostureResponse | null>(null);
  const [liveWorkflows, setLiveWorkflows] = useState<WorkflowSummary[]>([]);
  const [attestationData, setAttestationData] = useState<Record<string, unknown> | null>(null);
  const [liveManifest, setLiveManifest] = useState<EvidenceManifestResponse['manifest']>(null);
  const [verifyResult, setVerifyResult] = useState<AttestationVerifyResponse | null>(null);
  // provenance: null = connecting, true = live API reachable, false = unreachable
  const [apiReachable, setApiReachable] = useState<boolean | null>(null);
  const [workerRestartMessage, setWorkerRestartMessage] = useState<string | null>(null);
  // Vulnerability scan state
  const [scanData, setScanData] = useState<ScanResponse | null>(null);
  const [scanStatus, setScanStatus] = useState<'idle' | 'loading' | 'ready' | 'error'>('idle');
  const [scanError, setScanError] = useState<string | null>(null);

  useEffect(() => {
    const savedMode = window.sessionStorage.getItem('sentinel-experience');
    if (savedMode === 'live' || savedMode === 'replay') setExperienceMode(savedMode);
    setReviewerPrincipal(window.sessionStorage.getItem('sentinel-reviewer'));
  }, []);

  // Only contact the control plane after the user explicitly selects Live Cloud.
  useEffect(() => {
    if (experienceMode !== 'live') {
      setApiReachable(null);
      return;
    }

    setApiReachable(null);
    let isSubscribed = true;

    const checkHealth = () => {
      fetch(`${API_BASE}/v1/candidates?limit=1`, { headers: { 'X-Tenant-ID': TENANT_ID } })
        .then(r => {
          if (!isSubscribed) return;
          if (r.ok) {
            setApiReachable(true);
          } else {
            setTimeout(checkHealth, 2000);
          }
        })
        .catch(() => {
          if (!isSubscribed) return;
          setTimeout(checkHealth, 2000);
        });
    };

    checkHealth();

    return () => {
      isSubscribed = false;
    };
  }, [experienceMode]);

  // Fleet posture, workflow history, and registered candidate ABOMs are all
  // tenant runtime state. They stay empty unless the control plane returns them.
  const fetchFleetPosture = useCallback(async () => {
    try {
      const headers = { 'X-Tenant-ID': TENANT_ID };
      const [postureResponse, workflowsResponse, candidatesResponse] = await Promise.all([
        fetch(`${API_BASE}/v1/fleet/posture`, { headers }),
        fetch(`${API_BASE}/v1/workflows?limit=20`, { headers }),
        fetch(`${API_BASE}/v1/candidates?limit=25`, { headers }),
      ]);
      if (postureResponse.ok) setFleetPosture(await postureResponse.json() as FleetPostureResponse);
      if (workflowsResponse.ok) {
        const body = await workflowsResponse.json() as WorkflowListResponse;
        setLiveWorkflows(body.workflows);
      }
      if (candidatesResponse.ok) {
        const body = await candidatesResponse.json() as CandidateListResponse;
        setCandidates(body.candidates);
      }
    } catch (error) {
      setApiError(error instanceof Error ? error.message : String(error));
    }
  }, []);

  // Fetch vulnerability scan for the selected candidate revision.
  const fetchScan = useCallback(async (revisionId: string) => {
    setScanStatus('loading');
    setScanError(null);
    try {
      const headers = { 'X-Tenant-ID': TENANT_ID };
      const response = await fetch(`${API_BASE}/v1/candidates/${revisionId}/scan`, { headers });
      if (!response.ok) {
        throw new Error(`Scan request failed with HTTP ${response.status}`);
      }
      const data = await response.json() as ScanResponse;
      setScanData(data);
      setScanStatus('ready');
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setScanError(message);
      setScanStatus('error');
    }
  }, []);

  useEffect(() => {
    if (experienceMode !== 'live' || !apiReachable) return;
    // The approval tab needs the candidate ABOM to derive the capability
    // reduction, so load it there too.
    if (activeTab === 'fleet' || activeTab === 'candidate' || activeTab === 'approval') {
      void fetchFleetPosture();
    }
  }, [activeTab, apiReachable, experienceMode, fetchFleetPosture]);

  // Audit events belong to a specific durable workflow.
  useEffect(() => {
    if (experienceMode !== 'live' || !apiReachable || !liveWorkflowId) {
      setAuditEvents([]);
      return;
    }
    // The trace tab derives correlated trace IDs from these same events.
    if (activeTab !== 'certification' && activeTab !== 'trace') return;

    const controller = new AbortController();
    fetch(`${API_BASE}/v1/workflows/${liveWorkflowId}/audit?limit=100`, {
      headers: { 'X-Tenant-ID': TENANT_ID },
      signal: controller.signal,
    })
      .then(async response => {
        if (!response.ok) throw new Error(`Audit fetch failed: ${response.status}`);
        const body = await response.json() as AuditListResponse;
        setAuditEvents(body.events);
      })
      .catch((error: unknown) => {
        if (error instanceof DOMException && error.name === 'AbortError') return;
        setApiError(error instanceof Error ? error.message : String(error));
      });
    return () => controller.abort();
  }, [activeTab, apiReachable, experienceMode, liveWorkflowId, workflowState]);

  useEffect(() => {
    if (selectedCase) caseDialogRef.current?.focus();
  }, [selectedCase]);

  // Case definitions are served by the trusted core, which re-hashes every file
  // against corpus/v1/manifest.json before returning it. They are static
  // repository artifacts rather than run evidence, so they load in both
  // experience modes. Observed dispositions stay absent until a run produces
  // them, and no case is ever reconstructed in the browser.
  //
  // In guided walkthrough mode the control plane may not be reachable — that is expected.
  // We attempt the fetch regardless; if it fails we show an informational message
  // instead of a hard error, so the corpus tab is still usable as a read-only
  // reference once the control plane comes up.
  useEffect(() => {
    const controller = new AbortController();
    setCorpusStatus('loading');
    setCorpusError(null);

    fetch(`${API_BASE}/v1/corpus`, {
      headers: { 'X-Tenant-ID': TENANT_ID },
      signal: controller.signal,
    })
      .then(async response => {
        if (!response.ok) {
          throw new Error(
            response.status === 503
              ? 'The control plane is running but has not loaded corpus/v1.'
              : `Corpus request failed with HTTP ${response.status}.`,
          );
        }
        setCorpus(await response.json() as CorpusResponse);
        setCorpusStatus('ready');
      })
      .catch((error: unknown) => {
        if (error instanceof DOMException && error.name === 'AbortError') return;
        // In guided walkthrough mode a network failure is expected — the control
        // plane may be scaled to zero. Surface a softer message instead of a hard error.
        const message = error instanceof Error ? error.message : String(error);
        const isNetworkFailure = message.includes('502') || message.includes('fetch') || message.includes('Failed to fetch') || message.includes('NetworkError');
        if (experienceMode === 'replay' && isNetworkFailure) {
          setCorpusError('Corpus definitions load from the control plane. The control plane may be starting — switch to Live Cloud or wait for a cold start to browse the 80-case matrix.');
        } else {
          setCorpusError(message);
        }
        setCorpusStatus('error');
      });

    return () => controller.abort();
  }, [experienceMode]);

  const allCases = useMemo<TestCase[]>(() => corpus?.cases ?? [], [corpus]);

  // Latest durable workflow per candidate revision, so admission status is
  // derived from recorded state rather than assumed.
  const workflowByRevision = useMemo(() => {
    const map = new Map<string, WorkflowSummary>();
    for (const workflow of liveWorkflows) {
      const existing = map.get(workflow.candidate_revision_id);
      if (!existing || new Date(workflow.updated_at) > new Date(existing.updated_at)) {
        map.set(workflow.candidate_revision_id, workflow);
      }
    }
    return map;
  }, [liveWorkflows]);

  const isLiveConnected = experienceMode === 'live' && apiReachable === true;

  const selectedCandidate = useMemo(
    () => candidates.find(c => c.revision_id === selectedRevisionId) ?? candidates[0] ?? null,

    [candidates, selectedRevisionId],

  );



  // Load vulnerability scan when on the vulnerability tab and a candidate is selected.
  useEffect(() => {
    if (experienceMode !== 'live' || !apiReachable) return;
    if (activeTab === 'vulnerability' && selectedCandidate?.revision_id) {
      void fetchScan(selectedCandidate.revision_id);
    }
  }, [activeTab, apiReachable, experienceMode, fetchScan, selectedCandidate]);

  // Previous registered revision of the same agent, so the capability diff

  // compares two real ABOMs instead of an assumed baseline.

  const priorRevision = useMemo(() => {
    if (!selectedCandidate) return null;
    return (
      candidates
        .filter(
          c => c.agent_id === selectedCandidate.agent_id && c.revision_id !== selectedCandidate.revision_id,
        )
        .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())[0] ?? null
    );
  }, [candidates, selectedCandidate]);

  // Trace identifiers are grouped from recorded audit events, never synthesized.
  const correlatedTraces = useMemo(() => {
    const traces = new Map<string, { traceId: string; firstSeen: string; eventTypes: string[] }>();
    for (const event of auditEvents) {
      if (!event.trace_id) continue;
      const existing = traces.get(event.trace_id);
      if (existing) {
        if (!existing.eventTypes.includes(event.event_type)) {
          existing.eventTypes.push(event.event_type);
        }
        if (new Date(event.timestamp) < new Date(existing.firstSeen)) {
          existing.firstSeen = event.timestamp;
        }
      } else {
        traces.set(event.trace_id, {
          traceId: event.trace_id,
          firstSeen: event.timestamp,
          eventTypes: [event.event_type],
        });
      }
    }
    return Array.from(traces.values()).sort(
      (a, b) => new Date(a.firstSeen).getTime() - new Date(b.firstSeen).getTime(),
    );
  }, [auditEvents]);

  const revisionDiff = useMemo(() => {
    if (!selectedCandidate || !priorRevision) return null;
    const before = new Set(priorRevision.abom.requested_capabilities);
    const after = new Set(selectedCandidate.abom.requested_capabilities);
    const changedInputs = (
      [
        ['Source code', 'source_digest'],
        ['System prompt and generation config', 'prompt_config_digest'],
        ['Tool manifest', 'tool_manifest_digest'],
        ['Memory configuration', 'memory_config_digest'],
        ['Agent Gateway policy', 'gateway_policy_digest'],
        ['Model Armor template', 'model_armor_config_digest'],
      ] as const
    )
      .filter(([, key]) => priorRevision.abom[key] !== selectedCandidate.abom[key])
      .map(([label]) => label);

    return {
      added: Array.from(after).filter(capability => !before.has(capability)),
      removed: Array.from(before).filter(capability => !after.has(capability)),
      changedInputs,
      modelChanged: priorRevision.abom.model_ref !== selectedCandidate.abom.model_ref,
      riskChanged: priorRevision.abom.risk_tier !== selectedCandidate.abom.risk_tier,
    };
  }, [priorRevision, selectedCandidate]);

  const filteredCases = useMemo(() => {
    const needle = searchQuery.trim().toLowerCase();
    return allCases.filter(testCase => {
      const matchSearch =
        needle === '' ||
        testCase.case_id.toLowerCase().includes(needle) ||
        testCase.title.toLowerCase().includes(needle) ||
        testCase.category_name.toLowerCase().includes(needle);
      const matchSplit = splitFilter === 'all' || testCase.split === splitFilter;
      const matchCategory = categoryFilter === 'all' || testCase.category === categoryFilter;
      return matchSearch && matchSplit && matchCategory;
    });
  }, [allCases, searchQuery, splitFilter, categoryFilter]);

  const categoryOptions = useMemo(() => corpus?.categories ?? [], [corpus]);
  const corpusPageCount = Math.max(1, Math.ceil(filteredCases.length / CORPUS_PAGE_SIZE));
  const corpusStart = corpusPage * CORPUS_PAGE_SIZE;
  const paginatedCases = filteredCases.slice(corpusStart, corpusStart + CORPUS_PAGE_SIZE);

  useEffect(() => setCorpusPage(0), [searchQuery, splitFilter, categoryFilter]);

  useEffect(() => {
    if (experienceMode !== 'live' || !liveWorkflowId || activeTab !== 'evidence') return;

    const controller = new AbortController();
    fetch(`${API_BASE}/v1/workflows/${liveWorkflowId}/evidence`, {
      headers: { 'X-Tenant-ID': TENANT_ID },
      signal: controller.signal,
    })
      .then(async response => {
        if (!response.ok) throw new Error(`Evidence fetch failed: ${response.status}`);
        const body = await response.json() as EvidenceManifestResponse;
        setLiveManifest(body.manifest);
      })
      .catch(error => {
        if (error instanceof DOMException && error.name === 'AbortError') return;
        setApiError(error instanceof Error ? error.message : String(error));
      });
    return () => controller.abort();
  }, [activeTab, experienceMode, liveWorkflowId]);

  useEffect(() => {
    if (experienceMode !== 'live' || workflowState !== 'Certified' || !attestationData) return;

    const verificationAttestation = isTampered
      ? { ...attestationData, signature: 'INVALID_BASE64_SIG_MISMATCH==' }
      : attestationData;

    const controller = new AbortController();
    fetch(`${API_BASE}/v1/attestations/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'X-Tenant-ID': TENANT_ID },
      body: JSON.stringify({
        tenant_id: TENANT_ID,
        attestation: verificationAttestation,
        environment: 'production',
      }),
      signal: controller.signal,
    })
      .then(async response => {
        if (!response.ok) throw new Error(`Attestation verification failed: ${response.status}`);
        setVerifyResult(await response.json() as AttestationVerifyResponse);
      })
      .catch(error => {
        if (error instanceof DOMException && error.name === 'AbortError') return;
        setApiError(error instanceof Error ? error.message : String(error));
      });
    return () => controller.abort();
  }, [attestationData, experienceMode, isTampered, workflowState]);

  // Stepper helper
  const steps: WorkflowStep[] = [
    'Registered',
    'Queued',
    'Running',
    'EvidencePending',
    'ApprovalRequired',
    'RetestRequired',
    'Attesting',
    'Certified',
  ];

  const getStepIndex = (s: WorkflowStep) => steps.indexOf(s);
  const currentStepIdx = getStepIndex(workflowState);
  const isApprovalReady = workflowState === 'ApprovalRequired';
  const isCertified = workflowState === 'Certified';
  const isLiveCertified = isCertified && experienceMode === 'live' && apiReachable === true;
  const missionStatus = isCertified
    ? isLiveCertified
      ? 'Production authority earned'
      : 'Replay certification completed'
    : isApprovalReady
      ? 'Human decision required'
      : isRunningSim
        ? 'Autonomous evaluation running'
        : 'Candidate quarantined';
  const cloudProvenanceClass = experienceMode === 'live' && apiReachable
    ? 'provenance-live'
    : 'provenance-replay';
  const liveFleetPosture = experienceMode === 'live' && apiReachable ? fleetPosture : null;
  useEffect(() => {
    if (workflowState === 'ApprovalRequired') setActiveTab('approval');
    if (workflowState === 'Certified') setActiveTab('attestation');
  }, [workflowState]);
  const nextProof = isApprovalReady
    ? { tab: 'approval' as const, label: 'Review decision', icon: ArrowRight }
    : isCertified
      ? activeTab === 'attestation'
        ? { tab: 'trace' as const, label: 'Inspect trace', icon: Activity }
        : { tab: 'attestation' as const, label: 'Verify passport', icon: ShieldCheck }
      : workflowState === 'EvidencePending'
        ? { tab: 'evidence' as const, label: 'Inspect evidence', icon: Database }
        : null;

  // ── Live API Handlers ─────────────────────────────────────────────────────

  /** Runs a deterministic judge-friendly replay without claiming cloud provenance. */
  const handleStartReplay = useCallback(async () => {
    setIsRunningSim(true);
    setApiError(null);
    setApprovalGranted(false);
    setAttestationData(null);
    setLiveWorkflowId(`replay-${Date.now().toString(36)}`);

    const advance = async (state: WorkflowStep, delayMs: number) => {
      setWorkflowState(state);
      await new Promise(resolve => window.setTimeout(resolve, delayMs));
    };

    await advance('Queued', 450);
    await advance('Running', 1200);
    await advance('EvidencePending', 700);
    setWorkflowState('ApprovalRequired');
    setIsRunningSim(false);
  }, []);

  /** Creates a new workflow on the control plane and polls until ApprovalRequired. */
  const handleStartLiveCertification = useCallback(async () => {
    setIsRunningSim(true);
    setApiError(null);
    setWorkflowState('Queued');

    try {
      const candidatesRes = await fetch(`${API_BASE}/v1/candidates?limit=1`, {
        headers: { 'X-Tenant-ID': TENANT_ID },
      });
      if (!candidatesRes.ok) {
        throw new Error(`Candidate discovery failed: ${candidatesRes.status} ${await candidatesRes.text()}`);
      }
      const candidateList = await candidatesRes.json() as CandidateListResponse;
      const candidateRevisionId = candidateList.candidates[0]?.revision_id;
      if (!candidateRevisionId) {
        throw new Error('No registered candidate revision exists. Register a candidate revision before starting Live Cloud certification.');
      }

      const createRes = await fetch(`${API_BASE}/v1/workflows`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-Tenant-ID': TENANT_ID },
        body: JSON.stringify({
          tenant_id: TENANT_ID,
          candidate_revision_id: candidateRevisionId,
          policy_pack_id: 'ap-agent-v1',
          corpus_version: 'v1.0.0',
          idempotency_key: `web-${candidateRevisionId}-${Date.now()}`,
        }),
      });
      if (!createRes.ok) throw new Error(`Workflow create failed: ${createRes.status} ${await createRes.text()}`);
      const workflow = await createRes.json();
      const wfId: string = workflow.workflow_id;
      setLiveWorkflowId(wfId);
      setWorkflowState('Running');

      // 2. Poll until the workflow reaches ApprovalRequired (or terminal)
      let pollAttempts = 0;
      const poll = async (): Promise<void> => {
        pollAttempts += 1;
        if (pollAttempts > MAX_POLL_ATTEMPTS) {
          setApiError('Workflow has not progressed in 3 minutes. Check Cloud Run logs.');
          setIsRunningSim(false);
          return;
        }
        try {
          const pollRes = await fetch(`${API_BASE}/v1/workflows/${wfId}`, {
            headers: { 'X-Tenant-ID': TENANT_ID },
          });
          if (!pollRes.ok) throw new Error(`Poll failed: ${pollRes.status}`);
          const wf = await pollRes.json() as WorkflowApiResponse;
          const state = parseWorkflowState(wf.state);
          if (!state) throw new Error(`Workflow entered unsupported state: ${wf.state}`);
          setWorkflowState(state);
          if (
            state === 'ApprovalRequired' ||
            state === 'Certified' ||
            state === 'Failed' ||
            state === 'Blocked' ||
            state === 'Rejected' ||
            state === 'Cancelled' ||
            state === 'Revoked'
          ) {
            setIsRunningSim(false);
          } else {
            window.setTimeout(() => void poll(), 2000);
          }
        } catch (error) {
          setApiError(error instanceof Error ? error.message : String(error));
          setIsRunningSim(false);
        }
      };
      window.setTimeout(() => void poll(), 2000);
    } catch (err: unknown) {
      setApiError(err instanceof Error ? err.message : String(err));
      setIsRunningSim(false);
    }
  }, []);

  const handleStartCertification = useCallback(() => {
    if (experienceMode === 'replay') {
      void handleStartReplay();
      return;
    }
    if (!apiReachable) {
      setApiError('Live Cloud mode requires the Sentinel control plane. Verify the control plane is running, or switch to Guided Walkthrough mode.');
      return;
    }
    void handleStartLiveCertification();
  }, [apiReachable, experienceMode, handleStartLiveCertification, handleStartReplay]);

  /**
   * Discards console-held state and re-reads the workflow from the control
   * plane, so durability is demonstrated by recovery rather than asserted.
   */
  const handleDurabilityReread = useCallback(async () => {
    if (!liveWorkflowId) return;
    setApiError(null);
    setWorkerRestartMessage(null);

    setAttestationData(null);
    setLiveManifest(null);
    setAuditEvents([]);
    setWorkflowState('Registered');

    try {
      const headers = { 'X-Tenant-ID': TENANT_ID };
      const [workflowResponse, auditResponse] = await Promise.all([
        fetch(`${API_BASE}/v1/workflows/${liveWorkflowId}`, { headers }),
        fetch(`${API_BASE}/v1/workflows/${liveWorkflowId}/audit?limit=100`, { headers }),
      ]);
      if (!workflowResponse.ok) throw new Error(`Workflow re-read failed: ${workflowResponse.status}`);

      const workflow = await workflowResponse.json() as WorkflowApiResponse;
      const recovered = parseWorkflowState(workflow.state);
      if (!recovered) throw new Error(`Workflow returned unsupported state: ${workflow.state}`);
      setWorkflowState(recovered);

      let recoveredEvents = 0;
      if (auditResponse.ok) {
        const body = await auditResponse.json() as AuditListResponse;
        setAuditEvents(body.events);
        recoveredEvents = body.events.length;
      }

      setWorkerRestartMessage(
        `Console state discarded and re-read from durable storage: state ${workflow.state}, ${recoveredEvents} audit ${recoveredEvents === 1 ? 'event' : 'events'} recovered.`,
      );
    } catch (error) {
      setApiError(error instanceof Error ? error.message : String(error));
    }
  }, [liveWorkflowId]);

  /** Submits an approval decision to the control plane and polls to Certified */
  const handleGrantApproval = useCallback(async () => {
    setApprovalGranted(true);
    setApiError(null);

    if (experienceMode === 'replay') {
      setWorkflowState('RetestRequired');
      await new Promise(resolve => window.setTimeout(resolve, 650));
      setWorkflowState('Attesting');
      await new Promise(resolve => window.setTimeout(resolve, 650));
      const replayRevisionId = selectedCandidate?.revision_id ?? 'not-registered';
      const replayCapabilities = selectedCandidate
        ? selectedCandidate.abom.requested_capabilities.filter(c => !isElevatedCapability(c))
        : ['draft_invoice_payment', 'get_payment_status'];
      setAttestationData({
        schema_version: 'sentinel.attestation.v1',
        provenance: 'GUIDED_REPLAY',
        candidate_revision_id: replayRevisionId,
        decision: 'CERTIFIED_WITH_CONSTRAINTS',
        constrained_capabilities: replayCapabilities,
        note: 'This is a guided walkthrough envelope — not signed by Cloud KMS. Switch to Live Cloud for a real cryptographic attestation.',
      });
      setWorkflowState('Certified');
      return;
    }

    try {
      const wfId = liveWorkflowId;
      if (!wfId) throw new Error('No active workflow. Start certification first.');
      if (!reviewerPrincipal) {
        throw new Error(
          'No authenticated reviewer principal. Sign in with Google or GitHub so the decision can be attributed and separation of duties enforced.',
        );
      }
      if (!selectedCandidate) throw new Error('No candidate revision loaded for this decision.');

      // Derive the reduction from the candidate's real request so the reviewer
      // narrows actual requested authority rather than an assumed capability set.
      const requested = selectedCandidate.abom.requested_capabilities;
      const retained = requested.filter(capability => !isElevatedCapability(capability));
      const revoked = requested.filter(isElevatedCapability);

      const approvalRes = await fetch(`${API_BASE}/v1/workflows/${wfId}/approvals`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-Tenant-ID': TENANT_ID },
        body: JSON.stringify({
          tenant_id: TENANT_ID,
          reviewer: reviewerPrincipal,
          reviewer_role: reviewerRole,
          proposed_capabilities: requested,
          approved_capabilities: retained,
          reason_code: 'RULE-005-LEAST-PRIVILEGE-AGENCY',
          note: revoked.length > 0
            ? `Capability reduced: ${revoked.join(', ')} revoked.`
            : 'No elevated capability requested; authority retained as scoped.',
          duration_seconds: expiryDays * 24 * 60 * 60,
        }),
      });
      if (!approvalRes.ok) {
        const body = await approvalRes.text();
        throw new Error(`Approval failed (${approvalRes.status}): ${body}`);
      }
      setWorkflowState('Attesting');

      // Poll for Certified
      let pollAttempts = 0;
      const poll = async (): Promise<void> => {
        pollAttempts += 1;
        if (pollAttempts > MAX_POLL_ATTEMPTS) {
          setApiError('Attestation has not completed in 3 minutes. Check Cloud Run logs.');
          return;
        }
        try {
          const pollRes = await fetch(`${API_BASE}/v1/workflows/${wfId}`, {
            headers: { 'X-Tenant-ID': TENANT_ID },
          });
          if (!pollRes.ok) throw new Error(`Poll failed: ${pollRes.status}`);
          const wf = await pollRes.json() as WorkflowApiResponse;
          const state = parseWorkflowState(wf.state);
          if (!state) throw new Error(`Workflow entered unsupported state: ${wf.state}`);
          setWorkflowState(state);
          if (
            state === 'Failed' ||
            state === 'Blocked' ||
            state === 'Rejected' ||
            state === 'Cancelled' ||
            state === 'Revoked'
          ) {
            return; // Stop polling on failure
          }
          if (state !== 'Certified') {
            window.setTimeout(() => void poll(), 2000);
            return;
          }

          const attRes = await fetch(`${API_BASE}/v1/workflows/${wfId}/attestation`, {
            headers: { 'X-Tenant-ID': TENANT_ID },
          });
          if (!attRes.ok) throw new Error(`Attestation fetch failed: ${attRes.status}`);
          const body = await attRes.json() as AttestationResponse;
          if (!body.attestation) throw new Error('Certified workflow did not return an attestation.');
          setAttestationData(body.attestation);
        } catch (error) {
          setApiError(error instanceof Error ? error.message : String(error));
        }
      };
      window.setTimeout(() => void poll(), 2000);
    } catch (err: unknown) {
      setApiError(err instanceof Error ? err.message : String(err));
    }
  }, [
    experienceMode,
    liveWorkflowId,
    reviewerRole,
    expiryDays,
    reviewerPrincipal,
    selectedCandidate,
  ]);

  const handleExportReport = useCallback(() => {
    const header = ['Case ID', 'Category', 'Split', 'Expected Outcome', 'Model Armor (Expected)', 'Gateway (Expected)', 'Digest Verified'];
    const rows = allCases.map(tc => {
      return [
        tc.case_id,
        tc.category_name,
        tc.sealed ? 'Holdout' : 'Development',
        tc.expectations.expected_outcome,
        tc.expectations.expected_model_armor_disposition,
        tc.expectations.expected_gateway_disposition,
        tc.digest_verified ? 'VERIFIED' : 'MISMATCH'
      ].map(field => '"' + String(field).replace(/"/g, '""') + '"').join(',');
    });
    
    const csvContent = [header.join(','), ...rows].join('\n');
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `sentinel_compliance_audit_${new Date().toISOString().replace(/[:.]/g, '-')}.csv`;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 0);
  }, [allCases, verifyResult]);

    const contextValue = {
    activeTab,
    setActiveTab,
    experienceMode,
    setExperienceMode,
    workflowState,
    setWorkflowState,
    isRunningSim,
    setIsRunningSim,
    isTampered,
    setIsTampered,
    selectedCase,
    setSelectedCase,
    caseDialogRef,
    searchQuery,
    setSearchQuery,
    splitFilter,
    setSplitFilter,
    categoryFilter,
    setCategoryFilter,
    corpusPage,
    setCorpusPage,
    corpus,
    setCorpus,
    corpusStatus,
    setCorpusStatus,
    corpusError,
    setCorpusError,
    reviewerRole,
    setReviewerRole,
    approvalGranted,
    setApprovalGranted,
    expiryDays,
    setExpiryDays,
    reviewerPrincipal,
    setReviewerPrincipal,
    candidates,
    setCandidates,
    auditEvents,
    setAuditEvents,
    selectedRevisionId,
    setSelectedRevisionId,
    liveWorkflowId,
    setLiveWorkflowId,
    apiError,
    setApiError,
    fleetPosture,
    setFleetPosture,
    liveWorkflows,
    setLiveWorkflows,
    attestationData,
    setAttestationData,
    liveManifest,
    setLiveManifest,
    verifyResult,
    setVerifyResult,
    apiReachable,
    setApiReachable,
    allCases,
    workflowByRevision,
    isLiveConnected,
    selectedCandidate,
    priorRevision,
    correlatedTraces,
    revisionDiff,
    filteredCases,
    categoryOptions,
    corpusPageCount,
    corpusStart,
    paginatedCases,
    steps,
    currentStepIdx,
    isCertified,
    isApprovalReady,
    isLiveCertified,
    nextProof,
    missionStatus,
    cloudProvenanceClass,
    riskClass,
    shortDigest,
    auditSeverity,
    handleExportReport,
    handleStartCertification,
    liveFleetPosture,
    handleGrantApproval,
    handleDurabilityReread,
    workerRestartMessage,
    setWorkerRestartMessage,
    scanData,
    scanStatus,
    scanError,
    fetchScan,
    VERIFICATION_CHECKS,
    isElevatedCapability,
    CORPUS_PAGE_SIZE
  };

  return (
    <DashboardProvider value={contextValue}>
    <div className="flex h-screen overflow-hidden font-sans antialiased selection:bg-[rgba(0,212,255,0.2)]" style={{ background: 'var(--bg-primary)', color: 'var(--text-primary)' }}>

      <SidebarNav activeTab={activeTab} setActiveTab={setActiveTab} handleExportReport={handleExportReport} />

      <main className="flex-1 flex flex-col relative h-screen overflow-hidden min-w-0">

        {/* ── Top Action Bar ───────────────────────────────────────────────── */}
        <div
          className="flex-none h-[72px] px-6 flex items-center justify-between z-40 flex-shrink-0"
          style={{ borderBottom: '1px solid var(--border-subtle)', background: 'var(--bg-secondary)' }}
        >
          {/* Left: status pill + workflow ID + tenant */}
          <div className="flex items-center gap-4 min-w-0">
            <div className={cn(
              "flex items-center gap-2 px-3 py-1.5 rounded-full border text-[11px] font-bold tracking-wide flex-shrink-0",
              isCertified
                ? "bg-[rgba(0,228,155,0.1)] border-[rgba(0,228,155,0.3)] text-[#00e49b]"
                : isRunningSim
                  ? "bg-[rgba(0,212,255,0.08)] border-[rgba(0,212,255,0.25)] text-[#00d4ff]"
                  : "bg-[rgba(251,191,36,0.08)] border-[rgba(251,191,36,0.25)] text-[#fbbf24]"
            )}>
              <span className={cn(
                "w-1.5 h-1.5 rounded-full flex-shrink-0",
                isCertified   ? "bg-[#00e49b]" : "",
                isRunningSim  ? "bg-[#00d4ff] animate-pulse" : "",
                !isCertified && !isRunningSim ? "bg-[#fbbf24]" : ""
              )} style={isCertified ? { boxShadow: '0 0 6px rgba(0,228,155,0.8)' } : {}} />
              {missionStatus}
            </div>

            {liveWorkflowId && (
              <code
                className="px-2 py-1 rounded text-[11px] font-mono truncate max-w-[240px] hidden md:block flex-shrink-0"
                style={{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)', border: '1px solid var(--border-subtle)' }}
              >
                {liveWorkflowId}
              </code>
            )}

            <div className="hidden lg:flex items-center pl-4 flex-shrink-0" style={{ borderLeft: '1px solid var(--border-subtle)' }}>
              <span className="text-[10px] font-mono font-bold uppercase tracking-[0.1em]" style={{ color: 'var(--text-muted)' }}>
                tenant · {TENANT_ID.slice(0, 18)}…
              </span>
            </div>
          </div>

          {/* Right: action button */}
          <div className="flex items-center gap-3 flex-shrink-0">
            {nextProof ? (
              <button
                onClick={() => setActiveTab(nextProof.tab)}
                className={cn(
                  "flex items-center gap-2 px-4 py-2 rounded-lg text-[12.5px] font-bold tracking-wide transition-all duration-200",
                  isCertified
                    ? "text-[#021a0e]"
                    : "text-white"
                )}
                style={isCertified
                  ? { background: 'var(--accent-emerald)', boxShadow: '0 0 0 1px rgba(0,228,155,0.3), 0 4px 14px rgba(0,228,155,0.2)' }
                  : { background: '#4f46e5', boxShadow: '0 4px 14px rgba(79,70,229,0.3)' }
                }
              >
                {nextProof.label}
                <nextProof.icon size={14} />
              </button>
            ) : (
              <button
                onClick={handleStartCertification}
                disabled={isRunningSim}
                className="flex items-center gap-2 px-4 py-2 rounded-lg text-[12.5px] font-bold tracking-wide disabled:opacity-40 transition-all duration-200"
                style={{ background: 'var(--accent-emerald)', color: '#021a0e', boxShadow: '0 0 0 1px rgba(0,228,155,0.3), 0 4px 14px rgba(0,228,155,0.18)' }}
              >
                {isRunningSim ? <Activity className="animate-spin" size={14} /> : <Play size={14} />}
                {isRunningSim ? 'Evaluating…' : experienceMode === 'replay' ? 'Run Guided Replay' : 'Run Live Certification'}
              </button>
            )}
          </div>
        </div>

        {/* ── Global Error Bar ─────────────────────────────────────────────── */}
        <AnimatePresence>
          {apiError && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              className="inline-error-bar z-30 flex-shrink-0"
            >
              <span><strong>Error:</strong> {apiError}</span>
              <button onClick={() => setApiError(null)} style={{ flexShrink: 0 }}><X size={14} /></button>
            </motion.div>
          )}
        </AnimatePresence>

        {/* ── Scrollable Main Content ──────────────────────────────────────── */}
        <div className="flex-1 overflow-y-auto p-6 lg:p-8 relative custom-scrollbar min-h-0">
          <AnimatePresence mode="wait">
            <motion.div
              key={activeTab}
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -12 }}
              transition={{ duration: 0.2, ease: 'easeOut' }}
              className="max-w-[1400px] mx-auto"
            >
              {activeTab === 'fleet'         && <FleetView />}
              {activeTab === 'candidate'     && <CandidateView />}
              {activeTab === 'certification' && <CertificationView />}
              {activeTab === 'corpus'        && <CorpusView />}
              {activeTab === 'approval'      && <ApprovalView />}
              {activeTab === 'evidence'      && <EvidenceView />}
              {activeTab === 'attestation'   && <AttestationView />}
              {activeTab === 'trace'         && <TraceView />}
              {activeTab === 'policy'        && <PolicyView />}
              {activeTab === 'vulnerability' && <VulnerabilityView />}
            </motion.div>
          </AnimatePresence>
        </div>

      </main>
    </div>
    </DashboardProvider>
  );
}
