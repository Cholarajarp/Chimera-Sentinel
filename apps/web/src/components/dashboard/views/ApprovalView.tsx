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


export function ApprovalView(props: any) {
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
          <div id="approval-view">
            <div className="glass-panel action-banner">
              <div className="action-info">
                <h2>Human Governance & Constrained Approval Console</h2>
                <p>Enforce least-privilege capability reduction before release certification</p>
              </div>
              <div className="btn-group">
                <span className="provenance-tag provenance-demo">SEPARATION OF DUTIES: ENFORCED</span>
              </div>
            </div>

            <section className="approval-decision-packet" aria-labelledby="approval-packet-title">
              <div className="approval-packet-heading">
                <div>
                  <span>Review packet</span>
                  <h3 id="approval-packet-title">Authorize only the evidence-backed authority</h3>
                </div>
                <span className={`provenance-tag ${cloudProvenanceClass}`}>
                  {isLiveCertified || (experienceMode === 'live' && apiReachable) ? 'LIVE CONTROL PLANE' : 'GUIDED REPLAY'}
                </span>
              </div>
              {/* A reviewer's decision basis must be recorded fact, never a
                  placeholder. Unmeasured values stay visibly absent. */}
              <div className="approval-packet-grid">
                <div>
                  <span>Exact candidate</span>
                  <strong>
                    {selectedCandidate
                      ? selectedCandidate.agent_id
                      : isLiveConnected ? 'No candidate loaded' : 'Requires live connection'}
                  </strong>
                  {selectedCandidate && (
                    <code title={selectedCandidate.revision_id}>
                      {shortDigest(selectedCandidate.revision_id)}
                    </code>
                  )}
                </div>
                <div>
                  <span>Adversarial scope</span>
                  <strong>{corpus ? `${corpus.total_cases} cases` : '—'}</strong>
                  <small>
                    {corpus
                      ? `${corpus.categories.length} threat families · ${corpus.corpus_version}`
                      : 'Corpus definitions unavailable'}
                  </small>
                </div>
                <div>
                  <span>Business invariant</span>
                  <strong>{isCertified ? '$0.00 released' : 'Not yet proven'}</strong>
                  <small>
                    {isCertified
                      ? 'Unauthorized ledger delta'
                      : 'Ledger oracle result requires a completed run'}
                  </small>
                </div>
                <div>
                  <span>Required policy change</span>
                  <strong>Remove payment release</strong>
                  <small>Retest after approval</small>
                </div>
              </div>
              <div className="approval-packet-footer">
                <p>
                  {experienceMode === 'live' && apiReachable
                    ? 'This decision will be submitted to the live control plane and the constrained revision will be retested before attestation.'
                    : 'This replay demonstrates the review protocol. Switch to Live Cloud to submit a production decision backed by control-plane evidence.'}
                </p>
                <button className="btn btn-secondary" onClick={() => setActiveTab('evidence')}>
                  <Database size={15} aria-hidden="true" />
                  Inspect evidence
                </button>
              </div>
            </section>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(380px, 1fr))', gap: '1.5rem', marginBottom: '2rem' }}>
              <div className="glass-panel" style={{ padding: '1.75rem' }}>
                <h3 style={{ fontFamily: 'var(--font-display)', fontSize: '1.2rem', marginBottom: '1rem', color: 'var(--text-accent)' }}>
                  Proposed Least-Privilege Capability Diff
                </h3>
                <p style={{ color: 'var(--text-secondary)', fontSize: '0.85rem', marginBottom: '1.25rem' }}>
                  Candidate requested excessive autonomy (`release_payment`). Sentinel policy mandates narrowing capability to drafting only.
                </p>

                <div className="diff-container" style={{ margin: 0 }}>
                  <div className="diff-box diff-box-before">
                    <div style={{ fontWeight: 600, color: 'var(--accent-rose)', marginBottom: '0.5rem' }}>
                      REVOKED CAPABILITY
                    </div>
                    <span className="capability-pill-danger">release_payment</span>
                    <p style={{ color: 'var(--text-muted)', fontSize: '0.75rem', marginTop: '0.5rem' }}>
                      Prohibits direct release of financial funds
                    </p>
                  </div>

                  <div className="diff-box diff-box-after">
                    <div style={{ fontWeight: 600, color: 'var(--accent-emerald)', marginBottom: '0.5rem' }}>
                      APPROVED CAPABILITY
                    </div>
                    <span className="capability-pill-success">draft_invoice_payment</span>
                    <span className="capability-pill-success">get_payment_status</span>
                    <p style={{ color: 'var(--text-muted)', fontSize: '0.75rem', marginTop: '0.5rem' }}>
                      Retains useful invoice drafting and status querying
                    </p>
                  </div>
                </div>
              </div>

              <div className="glass-panel" style={{ padding: '1.75rem' }}>
                <h3 style={{ fontFamily: 'var(--font-display)', fontSize: '1.2rem', marginBottom: '1rem', color: 'var(--text-accent)' }}>
                  Reviewer Authorization Credentials
                </h3>

                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                  <div>
                    <label htmlFor="reviewer-principal" style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: '0.25rem' }}>
                      Reviewer principal (authenticated)
                    </label>
                    <input
                      id="reviewer-principal"
                      type="text"
                      className="search-input"
                      style={{ width: '100%' }}
                      value={reviewerPrincipal ?? 'Not signed in'}
                      readOnly
                      disabled
                    />
                    <p className="reviewer-principal-note">
                      {reviewerPrincipal
                        ? 'The control plane rejects this decision if the reviewer is the candidate owner.'
                        : 'Sign in with Google or GitHub to submit an attributable decision.'}
                    </p>
                  </div>

                  <div>
                    <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: '0.25rem' }}>
                      Reviewer RBAC Role
                    </label>
                    <select
                      className="filter-select"
                      style={{ width: '100%' }}
                      value={reviewerRole}
                      onChange={e => setReviewerRole(e.target.value)}
                    >
                      <option value="sec_reviewer">Security reviewer</option>
                      <option value="platform_owner">Platform owner</option>
                      <option value="compliance_officer">Compliance officer</option>
                    </select>
                    <p className="reviewer-principal-note">
                      Recorded with the decision. Separation of duties is enforced on identity, not
                      on this label.
                    </p>
                  </div>

                  <div>
                    <label style={{ display: 'block', fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: '0.25rem' }}>
                      Validity Window: {expiryDays} Days
                    </label>
                    <input
                      type="range"
                      min="30"
                      max="180"
                      value={expiryDays}
                      onChange={e => setExpiryDays(Number(e.target.value))}
                      style={{ width: '100%' }}
                    />
                  </div>

                  <div style={{ marginTop: '0.5rem' }}>
                    <button
                      className="btn btn-emerald"
                      id="btn-submit-approval"
                      style={{ width: '100%', justifyContent: 'center' }}
                      onClick={handleGrantApproval}
                      disabled={!isApprovalReady || approvalGranted}
                      title={!isApprovalReady ? 'Start a certification run first — workflow must be in ApprovalRequired state' : approvalGranted ? 'Approval already submitted' : undefined}
                    >
                      {approvalGranted ? 'Approval Submitted ✓' : 'Authorize Least-Privilege Policy & Dispatch Retest'}
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
    </>
  );
}
