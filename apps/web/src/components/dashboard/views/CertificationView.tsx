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


export function CertificationView(props: any) {
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
          <div id="timeline-view">
            <div className="panel action-banner">
              <div className="action-info">
                <h2>Durable Release Admission Workflow</h2>
                <p>Current Workflow State: <strong className="text-[#00e49b]">{workflowState}</strong></p>
              </div>
              <div className="btn-group">
                <button
                  className="btn btn-primary"
                  id="btn-start-certification"
                  onClick={handleStartCertification}
                  disabled={isRunningSim}
                >
                  {isRunningSim ? 'Running 80 Cases...' : 'Start Flagship Certification Run'}
                </button>
                <button
                  className="btn btn-secondary"
                  id="btn-restart-sim"
                  onClick={handleDurabilityReread}
                  disabled={!isLiveConnected || !liveWorkflowId}
                  title={
                    isLiveConnected && liveWorkflowId
                      ? 'Discard console state and re-read this workflow from durable storage'
                      : 'Requires a live workflow'
                  }
                >
                  Re-read from durable storage
                </button>
              </div>
            </div>

            {workerRestartMessage && (
              <div className="inline-success-bar" role="status">
                <span>{workerRestartMessage}</span>
                <button type="button" onClick={() => setWorkerRestartMessage(null)} aria-label="Dismiss">×</button>
              </div>
            )}

            {/* Stepper */}
            <div className="panel stepper-container">
              {steps.map((step, idx) => {
                const isCompleted = idx < currentStepIdx;
                const isActive = idx === currentStepIdx;
                return (
                  <div
                    key={step}
                    className={`step-item ${isCompleted ? 'completed' : ''} ${isActive ? 'active' : ''}`}
                  >
                    <div className="step-circle">
                      {isCompleted ? '✓' : idx + 1}
                    </div>
                    <span className="step-label">{step}</span>
                  </div>
                );
              })}
            </div>

            {/* Append-only audit trail, read back from durable storage. */}
            <div className="panel abom-panel">
              <div className="audit-head">
                <h4>Append-only workflow audit trail</h4>
                {isLiveConnected && liveWorkflowId && (
                  <span className="provenance-tag provenance-live">
                    {auditEvents.length} recorded {auditEvents.length === 1 ? 'event' : 'events'}
                  </span>
                )}
              </div>

              {!isLiveConnected || !liveWorkflowId ? (
                <p className="abom-panel-note">
                  {isLiveConnected
                    ? 'Start a certification run to record audit events for a workflow.'
                    : 'Audit events are written by the control plane. Switch to Live Cloud and start a run to read the real trail.'}
                </p>
              ) : auditEvents.length === 0 ? (
                <p className="abom-panel-note">
                  No audit events have been recorded for this workflow yet.
                </p>
              ) : (
                <ol className="audit-stream">
                  {auditEvents.map(event => (
                    <li key={event.event_id} className={`audit-event ${auditSeverity(event.event_type)}`}>
                      <div className="audit-event-head">
                        <span className="audit-event-type">{event.event_type}</span>
                        <time dateTime={event.timestamp}>{new Date(event.timestamp).toLocaleString()}</time>
                      </div>
                      <div className="audit-event-body">
                        {event.before_state && event.after_state && (
                          <span className="audit-transition">
                            {event.before_state} → {event.after_state}
                          </span>
                        )}
                        {event.decision && <strong>{event.decision}</strong>}
                        {event.explanation && <span>{event.explanation}</span>}
                      </div>
                      <div className="audit-event-meta">
                        <span>actor {event.actor}</span>
                        <span className="provenance-tag provenance-live">{event.provenance}</span>
                        {event.trace_id && <code title={event.trace_id}>trace {event.trace_id.slice(0, 16)}…</code>}
                      </div>
                    </li>
                  ))}
                </ol>
              )}
            </div>
          </div>
    </>
  );
}
