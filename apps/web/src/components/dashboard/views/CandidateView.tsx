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


export function CandidateView(props: any) {
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
          <div id="candidate-view">
            <div className="glass-panel action-banner">
              <div className="action-info">
                <h2>Agent Bill of Materials</h2>
                <p>
                  {selectedCandidate
                    ? `Immutable binding for ${selectedCandidate.agent_id} recorded ${new Date(selectedCandidate.abom.recorded_at).toLocaleString()}`
                    : 'Every security-critical input bound into one content-addressed manifest'}
                </p>
              </div>
              <div className="btn-group">
                {selectedCandidate && (
                  <span className="provenance-tag provenance-live" title={selectedCandidate.revision_id}>
                    {shortDigest(selectedCandidate.revision_id)}
                  </span>
                )}
              </div>
            </div>

            {!isLiveConnected && (
              <div className="glass-panel corpus-placeholder">
                <h3>ABOM requires a live connection</h3>
                <p>
                  An Agent Bill of Materials is a recorded artifact bound to a registered revision.
                </p>
                <p className="corpus-placeholder-note">
                  Switch to Live Cloud to load a real manifest. Sentinel will not show example
                  digests here, because an unverifiable digest proves nothing.
                </p>
              </div>
            )}

            {isLiveConnected && !selectedCandidate && (
              <div className="glass-panel corpus-placeholder">
                <h3>No candidate revisions registered</h3>
                <p>Register a candidate revision to inspect its Agent Bill of Materials.</p>
              </div>
            )}

            {isLiveConnected && selectedCandidate && (
              <>
                {candidates.length > 1 && (
                  <div className="glass-panel abom-selector">
                    <label htmlFor="abom-revision-select">Revision</label>
                    <select
                      id="abom-revision-select"
                      className="filter-select"
                      value={selectedCandidate.revision_id}
                      onChange={event => setSelectedRevisionId(event.target.value)}
                    >
                      {candidates.map(candidate => (
                        <option key={candidate.revision_id} value={candidate.revision_id}>
                          {candidate.agent_id} — {shortDigest(candidate.revision_id)}
                        </option>
                      ))}
                    </select>
                  </div>
                )}

                <div className="abom-grid">
                  <div className="glass-panel abom-panel">
                    <h4>Google Cloud managed resources</h4>
                    <div className="abom-fields">
                      <div>
                        <span>Agent Registry resource</span>
                        <code>{selectedCandidate.abom.registry_resource}</code>
                      </div>
                      <div>
                        <span>Agent Runtime resource</span>
                        <code>{selectedCandidate.abom.runtime_resource}</code>
                      </div>
                      <div>
                        <span>Dedicated agent identity</span>
                        <code className="is-identity">{selectedCandidate.abom.agent_identity}</code>
                      </div>
                      <div>
                        <span>Model reference</span>
                        <code className="is-model">{selectedCandidate.abom.model_ref}</code>
                      </div>
                      <div>
                        <span>Environment / owner</span>
                        <code>{selectedCandidate.abom.environment} · {selectedCandidate.abom.owner}</code>
                      </div>
                    </div>
                  </div>

                  <div className="glass-panel abom-panel">
                    <h4>Bound configuration digests</h4>
                    <p className="abom-panel-note">
                      Any change to these inputs produces a different revision, so an attestation
                      cannot follow a drifted configuration.
                    </p>
                    <div className="abom-fields">
                      {([
                        ['Source code', selectedCandidate.abom.source_digest],
                        ['System prompt and generation config', selectedCandidate.abom.prompt_config_digest],
                        ['Tool manifest', selectedCandidate.abom.tool_manifest_digest],
                        ['Memory configuration', selectedCandidate.abom.memory_config_digest],
                        ['Agent Gateway policy', selectedCandidate.abom.gateway_policy_digest],
                        ['Model Armor template', selectedCandidate.abom.model_armor_config_digest],
                      ] as const).map(([label, digest]) => (
                        <div key={label}>
                          <span>{label}</span>
                          <code className="is-digest">{digest}</code>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>

                <div className="abom-grid">
                  <div className="glass-panel abom-panel">
                    <h4>Requested capabilities</h4>
                    <div className="abom-capabilities">
                      {selectedCandidate.abom.requested_capabilities.map(capability => (
                        <span
                          key={capability}
                          className={
                            isElevatedCapability(capability)
                              ? 'capability-pill-danger'
                              : 'capability-pill-success'
                          }
                        >
                          {capability}
                        </span>
                      ))}
                    </div>
                  </div>

                  <div className="glass-panel abom-panel">
                    <h4>Risk and data classification</h4>
                    <div className="abom-capabilities">
                      <span className={`risk-tag ${riskClass(selectedCandidate.abom.risk_tier)}`}>
                        {selectedCandidate.abom.risk_tier.toUpperCase()} RISK
                      </span>
                      {selectedCandidate.abom.data_classification.map(classification => (
                        <span key={classification} className="split-tag">{classification}</span>
                      ))}
                    </div>
                    <div className="abom-fields" style={{ marginTop: '1rem' }}>
                      <div>
                        <span>Policy pack / corpus</span>
                        <code>{selectedCandidate.policy_pack_id} · {selectedCandidate.corpus_version}</code>
                      </div>
                    </div>
                  </div>
                </div>
              </>
            )}

            {isLiveConnected && selectedCandidate && (
              <div className="glass-panel abom-panel">
                <h4>Revision comparison</h4>
                {priorRevision && revisionDiff ? (
                  <>
                    <p className="abom-panel-note">
                      Structural diff against the previous registered revision of this agent.
                    </p>
                    <div className="diff-container">
                      <div className="diff-box diff-box-before">
                        <div className="diff-box-heading is-before">
                          Previous · {shortDigest(priorRevision.revision_id)}
                        </div>
                        <div className="abom-capabilities">
                          {priorRevision.abom.requested_capabilities.map(capability => (
                            <span key={capability} className="capability-pill-success">{capability}</span>
                          ))}
                        </div>
                      </div>

                      <div className="diff-box diff-box-after">
                        <div className="diff-box-heading is-after">
                          Candidate · {shortDigest(selectedCandidate.revision_id)}
                        </div>
                        <div className="abom-capabilities">
                          {selectedCandidate.abom.requested_capabilities.map(capability => (
                            <span
                              key={capability}
                              className={
                                revisionDiff.added.includes(capability)
                                  ? 'capability-pill-danger'
                                  : 'capability-pill-success'
                              }
                              title={revisionDiff.added.includes(capability) ? 'Newly requested authority' : undefined}
                            >
                              {capability}
                              {revisionDiff.added.includes(capability) ? ' (added)' : ''}
                            </span>
                          ))}
                        </div>
                      </div>
                    </div>

                    <dl className="case-expectations" style={{ marginTop: '1.25rem' }}>
                      <div>
                        <dt>Capabilities added</dt>
                        <dd><code>{revisionDiff.added.length > 0 ? revisionDiff.added.join(', ') : 'none'}</code></dd>
                      </div>
                      <div>
                        <dt>Capabilities removed</dt>
                        <dd><code>{revisionDiff.removed.length > 0 ? revisionDiff.removed.join(', ') : 'none'}</code></dd>
                      </div>
                      <div>
                        <dt>Changed security inputs</dt>
                        <dd><code>{revisionDiff.changedInputs.length > 0 ? revisionDiff.changedInputs.join(', ') : 'none'}</code></dd>
                      </div>
                      <div>
                        <dt>Model changed</dt>
                        <dd><code>{revisionDiff.modelChanged ? 'yes' : 'no'}</code></dd>
                      </div>
                    </dl>
                  </>
                ) : (
                  <p className="abom-panel-note">
                    No earlier revision of {selectedCandidate.agent_id} is registered, so there is
                    no baseline to diff against yet.
                  </p>
                )}
              </div>
            )}
          </div>
    </>
  );
}
