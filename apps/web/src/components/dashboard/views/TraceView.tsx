import React from 'react';
import { useDashboard } from '../DashboardContext';
import { motion, AnimatePresence } from 'framer-motion';
import { cn } from '@/lib/utils';
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
  Search,
  AlertTriangle
} from 'lucide-react';


export function TraceView(props: any) {
  const {
    activeTab, setActiveTab, experienceMode, setExperienceMode, workflowState, setWorkflowState,
    isRunningSim, setIsRunningSim, isTampered, setIsTampered, selectedCase, setSelectedCase,
    caseDialogRef, searchQuery, setSearchQuery, splitFilter, setSplitFilter, categoryFilter, setCategoryFilter,
    corpusPage, setCorpusPage, corpus, setCorpus, corpusStatus, setCorpusStatus, corpusError, setCorpusError,
    reviewerRole, setReviewerRole, approvalGranted, setApprovalGranted, expiryDays, setExpiryDays,
    reviewerPrincipal, setReviewerPrincipal, candidates, setCandidates, auditEvents, setAuditEvents,
    selectedRevisionId, setSelectedRevisionId, liveWorkflowId, setLiveWorkflowId, apiError, setApiError,
    fleetPosture, setFleetPosture, liveWorkflows, setLiveWorkflows, attestationData, setAttestationData,
    liveManifest, setLiveManifest, verifyResult, setVerifyResult, apiReachable, setApiReachable,
    allCases, workflowByRevision, isLiveConnected, selectedCandidate, priorRevision, correlatedTraces,
    revisionDiff, filteredCases, categoryOptions, corpusPageCount, corpusStart, paginatedCases, steps,
    currentStepIdx, isCertified, isApprovalReady, isLiveCertified, nextProof, missionStatus, cloudProvenanceClass,
    formatCurrency, riskClass, shortDigest, auditSeverity, handleExportReport, handleStartCertification,
    liveFleetPosture, handleCaseSimulation, handleOpenCase, handleStartPolicyDraft, handleExecuteManualApproval, handleGrantApproval, handleDurabilityReread, workerRestartMessage, setWorkerRestartMessage, VERIFICATION_CHECKS, isElevatedCapability, CORPUS_PAGE_SIZE
  } = useDashboard();

  return (
    <>
          <div id="trace-view">
            <div className="panel action-banner">
              <div className="action-info">
                <h2>Trace correlation</h2>
                <p>W3C trace identifiers recorded against durable workflow events</p>
              </div>
              <div className="btn-group">
                {correlatedTraces.length > 0 && (
                  <span className="provenance-tag provenance-live">
                    {correlatedTraces.length} correlated {correlatedTraces.length === 1 ? 'trace' : 'traces'}
                  </span>
                )}
              </div>
            </div>

            {/* Trace IDs are read from recorded audit events, never synthesized. */}
            <div className="panel abom-panel">
              <h4>Recorded trace identifiers</h4>
              {correlatedTraces.length === 0 ? (
                <p className="abom-panel-note">
                  {isLiveConnected && liveWorkflowId
                    ? 'No trace identifiers have been recorded against this workflow yet.'
                    : 'Trace identifiers are emitted by the instrumented services and stored with audit events. Switch to Live Cloud and run a certification to correlate a real trace.'}
                </p>
              ) : (
                <>
                  <p className="abom-panel-note">
                    Each identifier below was propagated through the control plane and written with
                    the event it belongs to, so a span can be opened in Cloud Trace.
                  </p>
                  <ol className="audit-stream">
                    {correlatedTraces.map(trace => (
                      <li key={trace.traceId} className="audit-event is-normal">
                        <div className="audit-event-head">
                          <code className="audit-event-type">{trace.traceId}</code>
                          <time dateTime={trace.firstSeen}>
                            {new Date(trace.firstSeen).toLocaleString()}
                          </time>
                        </div>
                        <div className="audit-event-body">
                          <span>
                            {trace.eventTypes.length} correlated{' '}
                            {trace.eventTypes.length === 1 ? 'event' : 'events'}: {trace.eventTypes.join(', ')}
                          </span>
                        </div>
                      </li>
                    ))}
                  </ol>
                </>
              )}

              <h4 className="mt-6">Telemetry redaction policy</h4>
              <p className="abom-panel-note">
                Spans are emitted through an allowlist so payloads stay out of telemetry. This states
                the policy; it is not a claim that an audit has been run in this session.
              </p>
              <ul className="verification-policy">
                <li><strong>Invoice bodies excluded</strong><span>Fixture payloads are never attached to spans</span></li>
                <li><strong>Prompt and completion text excluded</strong><span>Only dispositions and identifiers are exported</span></li>
                <li><strong>Credentials excluded</strong><span>Tokens and key material are never span attributes</span></li>
              </ul>
            </div>
          </div>
    </>
  );
}
