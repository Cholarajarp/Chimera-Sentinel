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
            <div className="panel action-banner">
              <div className="action-info">
                <h2>Human Governance & Constrained Approval Console</h2>
                <p>Enforce least-privilege capability reduction before release certification</p>
              </div>
              <div className="btn-group">
                <span className="provenance-tag provenance-live">SEPARATION OF DUTIES: ENFORCED</span>
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
                    : 'This guided walkthrough does not write to the control plane. Switch to Live Cloud to submit a production decision backed by control-plane evidence.'}
                </p>
                <button className="btn btn-secondary" onClick={() => setActiveTab('evidence')}>
                  <Database size={15} aria-hidden="true" />
                  Inspect evidence
                </button>
              </div>
            </section>

            <div className="grid gap-6 mb-6" style={{ gridTemplateColumns: 'repeat(auto-fit, minmax(360px, 1fr))' }}>
              {/* Capability diff — derived from the real candidate ABOM */}
              <div className="panel p-7">
                <h3 className="text-[17px] font-semibold mb-3" style={{ fontFamily: 'var(--font-display)', color: 'var(--text-accent)' }}>
                  Proposed Least-Privilege Capability Diff
                </h3>
                <p className="text-[13px] mb-5" style={{ color: 'var(--text-secondary)' }}>
                  {selectedCandidate
                    ? `${selectedCandidate.agent_id} requested ${selectedCandidate.abom.requested_capabilities.length} capabilities. Sentinel policy mandates removing elevated authority before release.`
                    : 'Capability reduction is derived from the registered candidate ABOM. Register a candidate and start a run to see the real diff.'}
                </p>

                {selectedCandidate ? (() => {
                  const revoked = selectedCandidate.abom.requested_capabilities.filter(isElevatedCapability);
                  const retained = selectedCandidate.abom.requested_capabilities.filter(c => !isElevatedCapability(c));
                  return (
                    <div className="diff-container" style={{ margin: 0 }}>
                      <div className="diff-box diff-box-before">
                        <div className="font-bold text-[11px] tracking-widest uppercase mb-2" style={{ color: 'var(--accent-rose)' }}>
                          Revoked ({revoked.length})
                        </div>
                        {revoked.length > 0
                          ? revoked.map(c => <span key={c} className="capability-pill-danger">{c}</span>)
                          : <span className="text-[12px]" style={{ color: 'var(--text-muted)' }}>No elevated capabilities requested</span>}
                      </div>
                      <div className="diff-box diff-box-after">
                        <div className="font-bold text-[11px] tracking-widest uppercase mb-2" style={{ color: 'var(--accent-emerald)' }}>
                          Approved ({retained.length})
                        </div>
                        {retained.length > 0
                          ? retained.map(c => <span key={c} className="capability-pill-success">{c}</span>)
                          : <span className="text-[12px]" style={{ color: 'var(--text-muted)' }}>No safe capabilities remain</span>}
                      </div>
                    </div>
                  );
                })() : (
                  <div className="diff-container" style={{ margin: 0 }}>
                    <div className="diff-box diff-box-before">
                      <div className="font-bold text-[11px] tracking-widest uppercase mb-2" style={{ color: 'var(--accent-rose)' }}>Revoked</div>
                      <span className="text-[12px]" style={{ color: 'var(--text-muted)' }}>Requires live candidate</span>
                    </div>
                    <div className="diff-box diff-box-after">
                      <div className="font-bold text-[11px] tracking-widest uppercase mb-2" style={{ color: 'var(--accent-emerald)' }}>Approved</div>
                      <span className="text-[12px]" style={{ color: 'var(--text-muted)' }}>Requires live candidate</span>
                    </div>
                  </div>
                )}
              </div>

              {/* Reviewer credentials */}
              <div className="panel p-7">
                <h3 className="text-[17px] font-semibold mb-4" style={{ fontFamily: 'var(--font-display)', color: 'var(--text-accent)' }}>
                  Reviewer Authorization Credentials
                </h3>

                <div className="flex flex-col gap-4">
                  <div>
                    <label htmlFor="reviewer-principal" className="block text-[11px] font-bold uppercase tracking-wider mb-1" style={{ color: 'var(--text-muted)' }}>
                      Reviewer principal (authenticated)
                    </label>
                    <input
                      id="reviewer-principal"
                      type="text"
                      className="search-input w-full"
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
                    <label className="block text-[11px] font-bold uppercase tracking-wider mb-1" style={{ color: 'var(--text-muted)' }}>
                      Reviewer RBAC Role
                    </label>
                    <select
                      className="filter-select w-full"
                      value={reviewerRole}
                      onChange={e => setReviewerRole(e.target.value)}
                    >
                      <option value="sec_reviewer">Security reviewer</option>
                      <option value="platform_owner">Platform owner</option>
                      <option value="compliance_officer">Compliance officer</option>
                    </select>
                    <p className="reviewer-principal-note">
                      Recorded with the decision. Separation of duties is enforced on identity, not on this label.
                    </p>
                  </div>

                  <div>
                    <label className="block text-[11px] font-bold uppercase tracking-wider mb-1" style={{ color: 'var(--text-muted)' }}>
                      Validity Window: {expiryDays} Days
                    </label>
                    {/* Thin progress bar showing validity */}
                    <div className="progress-bar-track mb-2">
                      <div className="progress-bar-fill" style={{ width: `${((expiryDays - 30) / 150) * 100}%` }} />
                    </div>
                    <input
                      type="range"
                      min="30"
                      max="180"
                      value={expiryDays}
                      onChange={e => setExpiryDays(Number(e.target.value))}
                      className="w-full accent-[#00e49b]"
                    />
                  </div>

                  <div className="mt-1">
                    <button
                      className="btn btn-emerald w-full justify-center"
                      id="btn-submit-approval"
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
