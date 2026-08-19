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


export function AttestationView(props: any) {
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
          <div id="attestation-view">
            <div className="glass-panel action-banner">
              <div className="action-info">
                <h2>Cloud KMS Attestation Verifier</h2>
                <p>The control plane verifies the signed envelope and returns one verdict</p>
              </div>
              <div className="btn-group">
                <button
                  className="btn btn-secondary"
                  id="btn-tamper-sim"
                  onClick={() => setIsTampered(!isTampered)}
                  disabled={!attestationData || !isLiveConnected}
                  title={
                    !attestationData
                      ? 'Requires an issued attestation'
                      : isLiveConnected
                        ? 'Corrupt the signature and submit it to the control-plane verifier'
                        : 'Tamper rejection is proven by the control plane, so it requires Live Cloud'
                  }
                >
                  {isTampered ? 'Restore original payload' : 'Corrupt payload and re-verify'}
                </button>
              </div>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(360px, 1fr))', gap: '1.5rem', marginBottom: '2rem' }}>
              {/* The verifier returns one aggregate verdict, so exactly one
                  verdict is displayed. Individual checks are never self-marked. */}
              <div className="glass-panel abom-panel">
                <h4>Verification verdict</h4>

                {verifyResult ? (
                  <div className={`verification-result ${verifyResult.valid ? 'is-valid' : 'is-invalid'}`} role="status">
                    <strong>
                      {verifyResult.valid
                        ? 'Signature verified by the control plane'
                        : 'Verification rejected by the control plane'}
                    </strong>
                    <span>{verifyResult.details}</span>
                  </div>
                ) : (
                  <p className="abom-panel-note">
                    {!attestationData
                      ? 'No attestation has been issued yet. Complete a certification run to produce a signed envelope, then it is verified here.'
                      : isLiveConnected
                        ? 'Submitting the envelope to the control-plane verifier…'
                        : 'This guided walkthrough envelope is not cryptographically verified by the control plane. Switch to Live Cloud to verify a real Cloud KMS signature.'}
                  </p>
                )}

                {isTampered && (
                  <p className="tamper-note" role="status">
                    The payload has been deliberately corrupted. A valid verdict here would indicate
                    a broken verifier.
                  </p>
                )}

                <h4 style={{ marginTop: '1.5rem' }}>Checks covered by this verdict</h4>
                <p className="abom-panel-note">
                  The verifier evaluates all of the following and fails closed on any one of them.
                </p>
                <ol className="verification-policy">
                  {VERIFICATION_CHECKS.map(check => (
                    <li key={check.id}>
                      <strong>{check.label}</strong>
                      <span>{check.detail}</span>
                    </li>
                  ))}
                </ol>
              </div>

              {/* Envelope JSON Viewer — live data if certified, walkthrough otherwise */}
              <div className="glass-panel" style={{ padding: '1.75rem' }}>
                <h4 style={{ fontFamily: 'var(--font-display)', fontSize: '1.2rem', marginBottom: '1rem', color: 'var(--text-accent)' }}>
                  Canonical Attestation Envelope
                  {attestationData && (
                    <span
                      className={`provenance-tag ${experienceMode === 'live' ? 'provenance-live' : 'provenance-replay'}`}
                      style={{ marginLeft: '0.75rem', fontSize: '0.7rem' }}
                    >
                      {experienceMode === 'live' ? 'LIVE CLOUD' : 'GUIDED REPLAY'}
                    </span>
                  )}
                </h4>
                {attestationData ? (
                  <>
                    {isTampered && (
                      <p className="tamper-note">
                        Showing the corrupted payload exactly as submitted for verification.
                      </p>
                    )}
                    <pre className={`attestation-envelope ${isTampered ? 'is-tampered' : ''}`}>
                      {JSON.stringify(
                        isTampered
                          ? { ...attestationData, signature: 'INVALID_BASE64_SIG_MISMATCH==' }
                          : attestationData,
                        null,
                        2,
                      )}
                    </pre>
                  </>
                ) : (
                  <p className="abom-panel-note">
                    No envelope has been issued. A signed attestation exists only after a run
                    reaches Certified, so none is shown here.
                  </p>
                )}
              </div>
            </div>
          </div>
    </>
  );
}
