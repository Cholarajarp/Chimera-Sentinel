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


export function FleetView(props: any) {
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
          <div id="fleet-view">
            <section className="mission-control" aria-labelledby="mission-title">
              <div className="mission-primary">
                <div className="mission-eyebrow">
                  <span className="mission-pulse" aria-hidden="true" />
                  {selectedCandidate
                    ? `Release gate / ${selectedCandidate.agent_id} / ${selectedCandidate.abom.environment}`
                    : 'Release gate / AI agent admission control'}
                </div>
                <div className="mission-title-row">
                  <h1 id="mission-title">Production access is a security decision.</h1>
                  <span className={`mission-state ${isCertified ? 'is-certified' : isApprovalReady ? 'is-review' : ''}`}>
                    {isCertified
                      ? isLiveCertified ? 'Authority earned' : 'Replay complete'
                      : isApprovalReady ? 'Reviewer action' : 'Quarantined'}
                  </span>
                </div>
                <p className="mission-summary">
                  Sentinel binds the exact deployable revision, attacks it across hostile workflows, verifies the
                  ledger outcome, and grants only the authority that survives the evidence.
                </p>

                <div className="mission-command-row">
                  {nextProof ? (
                    <button
                      className={`mission-command ${isCertified ? 'is-certified' : ''}`}
                      onClick={() => setActiveTab(nextProof.tab)}
                    >
                      <nextProof.icon size={17} aria-hidden="true" />
                      {nextProof.label}
                      <ArrowRight size={16} aria-hidden="true" />
                    </button>
                  ) : (
                    <button
                      className="mission-command"
                      onClick={handleStartCertification}
                      disabled={isRunningSim}
                    >
                      {isRunningSim ? <Activity className="spinner" size={17} aria-hidden="true" /> : <Play size={17} aria-hidden="true" />}
                      {isRunningSim ? 'Testing the candidate' : 'Run the release gate'}
                      {!isRunningSim && <ArrowRight size={16} aria-hidden="true" />}
                    </button>
                  )}
                  <button className="mission-inspect" onClick={() => setActiveTab('candidate')}>
                    <Fingerprint size={16} aria-hidden="true" />
                    Inspect immutable ABOM
                  </button>
                </div>

                <div className="evidence-snapshot" aria-label="Release gate evidence summary">
                  <div className="evidence-snapshot-head">
                    <span>Evidence signal</span>
                    <span className={`provenance-tag ${cloudProvenanceClass}`}>{experienceMode === 'live' && apiReachable ? 'LIVE CONTROL PLANE' : 'GUIDED REPLAY'}</span>
                  </div>
                  {/* Corpus scope is a manifest fact; outcomes require a run. */}
                  <div className="evidence-readouts">
                    <div>
                      <strong>{corpus ? corpus.total_cases : '—'}</strong>
                      <span>adversarial cases in scope</span>
                    </div>
                    <div>
                      <strong>{corpus ? corpus.categories.length : '—'}</strong>
                      <span>threat families</span>
                    </div>
                    <div>
                      <strong>{isCertified ? '$0.00' : '—'}</strong>
                      <span>{isCertified ? 'unauthorized ledger delta' : 'ledger delta pending run'}</span>
                    </div>
                  </div>
                  {experienceMode === 'replay' && (
                    <p className="replay-disclosure">
                      Guided replay illustrates the release path. It does not grant production authority or replace live control-plane evidence.
                    </p>
                  )}
                </div>

                <div className="experience-switch" role="group" aria-label="Mission Control data source">
                  <button
                    type="button"
                    className={experienceMode === 'replay' ? 'active' : ''}
                    aria-pressed={experienceMode === 'replay'}
                    onClick={() => {
                        setExperienceMode('replay');
                        window.sessionStorage.setItem('sentinel-experience', 'replay');
                        setApiError(null);
                        setWorkflowState('Registered');
                        setLiveWorkflowId(null);
                        setApprovalGranted(false);
                        setAttestationData(null);
                        setVerifyResult(null);
                        setLiveManifest(null);
                      }}
                  >
                    Guided Replay
                    <span>Deterministic judge walkthrough</span>
                  </button>
                  <button
                    type="button"
                    className={experienceMode === 'live' ? 'active' : ''}
                    aria-pressed={experienceMode === 'live'}
                    onClick={() => {
                        setExperienceMode('live');
                        window.sessionStorage.setItem('sentinel-experience', 'live');
                        setApiError(null);
                        setWorkflowState('Registered');
                        setLiveWorkflowId(null);
                        setApprovalGranted(false);
                        setAttestationData(null);
                        setVerifyResult(null);
                        setLiveManifest(null);
                      }}
                  >
                    Live Cloud
                    <span>{apiReachable ? 'Control plane connected' : 'Requires API connection'}</span>
                  </button>
                </div>
              </div>

              <aside className="mission-decision" aria-label="Current admission decision">
                <div className="decision-heading">
                  <div>
                    <span>Authority surface</span>
                    <strong>What this agent may do</strong>
                  </div>
                  <span className={`connection-state ${apiReachable ? 'is-live' : ''}`}>
                    <Cloud size={14} aria-hidden="true" />
                    {experienceMode === 'replay'
                      ? 'Illustrative replay'
                      : apiReachable === null
                        ? 'Connecting'
                        : apiReachable
                          ? 'Cloud API live'
                          : 'Cloud API offline'}
                  </span>
                </div>

                <div className="authority-diff">
                  <div className="authority-requested">
                    <span>Requested</span>
                    <code>release_payment</code>
                    <small>Directly moves funds</small>
                  </div>
                  <ArrowRight className="authority-arrow" size={18} aria-hidden="true" />
                  <div className="authority-constrained">
                    <span>Allowed after review</span>
                    <code>draft_invoice_payment</code>
                    <small>Prepares, never releases</small>
                  </div>
                </div>

                <div className="decision-rationale">
                  <LockKeyhole size={18} aria-hidden="true" />
                  <p><strong>Least privilege wins.</strong> The payment-release capability remains outside the signed production passport.</p>
                </div>

                <ol className="control-path">
                  <li className={currentStepIdx >= 0 ? 'complete' : ''}>
                    <CheckCircle2 size={17} aria-hidden="true" />
                    <div><strong>1. Bind</strong><span>Revision, identity, policy</span></div>
                  </li>
                  <li className={currentStepIdx >= 2 ? 'complete' : 'pending'}>
                    <Activity size={17} aria-hidden="true" />
                    <div><strong>2. Attack</strong><span>ADK, Model Armor, Gateway</span></div>
                  </li>
                  <li className={currentStepIdx >= 3 ? 'complete' : 'pending'}>
                    <Database size={17} aria-hidden="true" />
                    <div><strong>3. Verify</strong><span>Independent ERP ledger oracle</span></div>
                  </li>
                  <li className={isCertified ? 'complete' : 'pending'}>
                    <ShieldCheck size={17} aria-hidden="true" />
                    <div><strong>4. Attest</strong><span>Expiring Cloud KMS passport</span></div>
                  </li>
                </ol>

                {liveWorkflowId && (
                  <div className="workflow-reference">
                    <span>Workflow</span>
                    <code>{liveWorkflowId}</code>
                  </div>
                )}
              </aside>
            </section>

            {/* KPI Cards */}
            <div className="kpi-grid">
              <div className={`glass-panel kpi-card ${
                liveFleetPosture
                  ? liveFleetPosture.posture === 'HEALTHY' ? 'posture-healthy' : 'posture-attention'
                  : ''
              }`}>
                <div className="kpi-card-header">
                  <span>Workflow Posture</span>
                  <span className={`provenance-tag ${liveFleetPosture ? 'provenance-live' : 'provenance-local'}`}>
                    {liveFleetPosture ? 'LIVE' : 'NO DATA'}
                  </span>
                </div>
                {/* Never assert a healthy posture without control-plane evidence. */}
                <div
                  className={`kpi-value ${liveFleetPosture ? '' : 'is-unknown'}`}
                  style={liveFleetPosture ? { color: 'var(--accent-emerald)' } : undefined}
                >
                  {liveFleetPosture ? liveFleetPosture.posture.replace('_', ' ') : 'UNKNOWN'}
                </div>
                <div className="kpi-subtext">
                  {liveFleetPosture
                    ? 'Derived from durable certification workflow states'
                    : 'Posture is unproven until the control plane reports workflow state'}
                </div>
              </div>

              <div className="glass-panel kpi-card">
                <div className="kpi-card-header">
                  <span>Certification Workflows</span>
                </div>
                <div className={`kpi-value ${liveFleetPosture ? '' : 'is-unknown'}`}>
                  {liveFleetPosture?.total_workflows ?? '—'}
                </div>
                <div className="kpi-subtext">
                  {liveFleetPosture
                    ? 'Total durable runs for this tenant'
                    : 'Requires a live control-plane connection'}
                </div>
              </div>

              <div className="glass-panel kpi-card">
                <div className="kpi-card-header">
                  <span>Certified</span>
                </div>
                <div
                  className={`kpi-value ${liveFleetPosture ? '' : 'is-unknown'}`}
                  style={liveFleetPosture ? { color: 'var(--accent-indigo)' } : undefined}
                >
                  {liveFleetPosture?.certified ?? '—'}
                </div>
                <div className="kpi-subtext">
                  {liveFleetPosture
                    ? 'Runs that completed policy and attestation'
                    : 'Requires a live control-plane connection'}
                </div>
              </div>

              <div className="glass-panel kpi-card">
                <div className="kpi-card-header">
                  <span>Running / Review</span>
                </div>
                <div
                  className={`kpi-value ${liveFleetPosture ? '' : 'is-unknown'}`}
                  style={liveFleetPosture ? { color: 'var(--accent-amber)' } : undefined}
                >
                  {liveFleetPosture
                    ? liveFleetPosture.running + liveFleetPosture.approval_required
                    : '—'}
                </div>
                <div className="kpi-subtext">
                  {liveFleetPosture
                    ? 'Active execution or awaiting least-privilege approval'
                    : 'Requires a live control-plane connection'}
                </div>
              </div>

              <div className={`glass-panel kpi-card ${(liveFleetPosture?.blocked ?? 0) > 0 ? 'posture-blocked' : ''}`}>
                <div className="kpi-card-header">
                  <span>Blocked Workflows</span>
                </div>
                <div
                  className={`kpi-value ${liveFleetPosture ? '' : 'is-unknown'}`}
                  style={liveFleetPosture ? { color: liveFleetPosture.blocked ? 'var(--accent-rose)' : 'var(--accent-emerald)' } : undefined}
                >
                  {liveFleetPosture?.blocked ?? '—'}
                </div>
                <div className="kpi-subtext">
                  {liveFleetPosture
                    ? 'Fail-closed terminal admission decisions'
                    : 'Requires a live control-plane connection'}
                </div>
              </div>
            </div>

            {liveFleetPosture && (
              <div className="glass-panel" style={{ padding: '1.5rem', marginBottom: '2rem' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.25rem' }}>
                  <div>
                    <h3 style={{ fontFamily: 'var(--font-display)', fontSize: '1.15rem' }}>Recent Certification Workflows</h3>
                    <p style={{ color: 'var(--text-secondary)', fontSize: '0.8125rem' }}>Durable workflow history returned by the control plane</p>
                  </div>
                  <span className="provenance-tag provenance-live">LIVE</span>
                </div>
                <div className="table-wrapper">
                  <table className="sentinel-table">
                    <thead><tr><th>Workflow</th><th>Candidate Revision</th><th>State</th><th>Decision</th><th>Updated</th></tr></thead>
                    <tbody>
                      {liveWorkflows.length > 0 ? liveWorkflows.map(workflow => (
                        <tr key={workflow.workflow_id}>
                          <td><code>{workflow.workflow_id}</code></td>
                          <td><code>{workflow.candidate_revision_id}</code></td>
                          <td><span className="provenance-tag provenance-live">{workflow.state}</span></td>
                          <td>{workflow.gate_decision ?? 'Pending'}</td>
                          <td>{new Date(workflow.updated_at).toLocaleString()}</td>
                        </tr>
                      )) : <tr><td colSpan={5}>No workflows exist for this tenant yet.</td></tr>}
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {/* Fleet Inventory Table */}
            <div className="glass-panel" style={{ padding: '1.5rem', marginBottom: '2rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.25rem' }}>
                <div>
                  <h3 style={{ fontFamily: 'var(--font-display)', fontSize: '1.15rem' }}>
                    Registered Agent Revisions
                  </h3>
                  <p style={{ color: 'var(--text-secondary)', fontSize: '0.8125rem' }}>
                    Immutable candidate revisions recorded by the control plane
                  </p>
                </div>
                <button className="btn btn-primary" id="btn-certify-top" onClick={() => setActiveTab('certification')}>
                  Certify New Revision
                </button>
              </div>

              {isLiveConnected ? (
                <div className="table-wrapper">
                  <table className="sentinel-table">
                    <thead>
                      <tr>
                        <th>Agent identifier</th>
                        <th>Revision digest</th>
                        <th>Model reference</th>
                        <th>Agent identity principal</th>
                        <th>Risk tier</th>
                        <th>Admission status</th>
                        <th>Provenance</th>
                      </tr>
                    </thead>
                    <tbody>
                      {candidates.length > 0 ? (
                        candidates.map(candidate => {
                          const workflow = workflowByRevision.get(candidate.revision_id);
                          return (
                            <tr key={candidate.revision_id}>
                              <td>
                                <strong>{candidate.agent_id}</strong>
                                <div className="case-subtitle">
                                  {candidate.abom.environment} · {candidate.policy_pack_id}
                                </div>
                              </td>
                              <td>
                                <code className="digest-cell" title={candidate.revision_id}>
                                  {shortDigest(candidate.revision_id)}
                                </code>
                              </td>
                              <td>{candidate.abom.model_ref}</td>
                              <td>
                                <code className="digest-cell" title={candidate.abom.agent_identity}>
                                  {candidate.abom.agent_identity}
                                </code>
                              </td>
                              <td>
                                <span className={`risk-tag ${riskClass(candidate.abom.risk_tier)}`}>
                                  {candidate.abom.risk_tier.toUpperCase()}
                                </span>
                              </td>
                              <td>
                                {workflow ? (
                                  <span className="provenance-tag provenance-live">{workflow.state}</span>
                                ) : (
                                  <span className="provenance-tag provenance-local">NOT SUBMITTED</span>
                                )}
                              </td>
                              <td>
                                <span className="provenance-tag provenance-live">{candidate.abom.provenance}</span>
                              </td>
                            </tr>
                          );
                        })
                      ) : (
                        <tr>
                          <td colSpan={7}>No candidate revisions are registered for this tenant yet.</td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              ) : (
                <div className="corpus-placeholder">
                  <h3>Fleet inventory requires a live connection</h3>
                  <p>
                    Registered revisions, their identities, and admission status are tenant state
                    owned by the control plane.
                  </p>
                  <p className="corpus-placeholder-note">
                    Switch to Live Cloud to load the real inventory. Sentinel does not display
                    sample agents in place of registered ones.
                  </p>
                </div>
              )}
            </div>
          </div>
    </>
  );
}
