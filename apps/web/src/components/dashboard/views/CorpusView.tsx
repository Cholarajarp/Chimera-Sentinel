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


export function CorpusView(props: any) {
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
          <div id="corpus-view">
            <div className="panel action-banner">
              <div className="action-info">
                <h2>{corpus ? corpus.name : 'Adversarial Evaluation Corpus'}</h2>
                <p>
                  {corpus
                    ? `${corpus.corpus_version} · ${corpus.development_cases} development and ${corpus.holdout_cases} sealed holdout cases across ${corpus.categories.length} threat families`
                    : 'Versioned case definitions served by the Rust control plane'}
                </p>
              </div>
              <div className="btn-group">
                {corpus && (
                  <span
                    className={`provenance-tag ${corpus.integrity.all_digests_verified ? 'provenance-live' : 'provenance-warn'}`}
                    title={`Manifest ${corpus.integrity.manifest_sha256}`}
                  >
                    {corpus.integrity.all_digests_verified
                      ? `${corpus.integrity.digest_verified_cases}/${corpus.integrity.declared_cases} digests verified`
                      : `Integrity incomplete: ${corpus.integrity.digest_verified_cases}/${corpus.integrity.declared_cases}`}
                  </span>
                )}
              </div>
            </div>

            {corpusStatus === 'loading' && (
              <div className="panel corpus-placeholder" role="status">
                <Activity size={18} className="spinner" aria-hidden="true" />
                <p>Loading digest-verified case definitions from the control plane…</p>
              </div>
            )}

            {corpusStatus === 'error' && (
              <div className="panel corpus-placeholder is-error" role="alert">
                <h3>Case definitions unavailable</h3>
                <p>{corpusError}</p>
                <p className="corpus-placeholder-note">
                  Sentinel does not reconstruct evaluation cases in the browser. Start the control
                  plane with corpus/v1 available to inspect the real corpus.
                </p>
              </div>
            )}

            {/* Filters Row */}
            {corpusStatus === 'ready' && (
            <div className="panel mb-6">
              <div className="filter-row">
                <input
                  id="case-search-input"
                  type="text"
                  placeholder="Search by case ID, title, or category..."
                  className="search-input"
                  value={searchQuery}
                  onChange={e => setSearchQuery(e.target.value)}
                />

                <div className="flex gap-3">
                  <select
                    id="split-filter-select"
                    className="filter-select"
                    value={splitFilter}
                    onChange={e => setSplitFilter(e.target.value as 'all' | 'development' | 'holdout')}
                  >
                    <option value="all">All splits ({corpus?.total_cases ?? 0})</option>
                    <option value="development">Development ({corpus?.development_cases ?? 0})</option>
                    <option value="holdout">Sealed holdout ({corpus?.holdout_cases ?? 0})</option>
                  </select>

                  <select
                    id="category-filter-select"
                    className="filter-select"
                    value={categoryFilter}
                    onChange={e => setCategoryFilter(e.target.value)}
                  >
                    <option value="all">All threat families ({categoryOptions.length})</option>
                    {categoryOptions.map(category => (
                      <option key={category.id} value={category.id}>
                        {category.name} ({category.total})
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              {/* Table */}
              <div className="table-wrapper">
                <table className="sentinel-table" id="cases-table">
                  <thead>
                    <tr>
                      <th>Case identifier</th>
                      <th>Threat family</th>
                      <th>Split</th>
                      <th>Required outcome</th>
                      <th>Model Armor (expected)</th>
                      <th>Gateway (expected)</th>
                      <th>Manifest digest</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {paginatedCases.map(testCase => (
                      <tr key={testCase.case_id}>
                        <td>
                          <code className="case-id">{testCase.case_id}</code>
                          <div className="case-subtitle">{testCase.title}</div>
                        </td>
                        <td>{testCase.category_name}</td>
                        <td>
                          <span className={`split-tag ${testCase.sealed ? 'is-sealed' : ''}`}>
                            {testCase.sealed ? 'SEALED HOLDOUT' : 'DEVELOPMENT'}
                          </span>
                        </td>
                        <td>
                          <code className="outcome-code">{testCase.expectations.expected_outcome}</code>
                        </td>
                        <td>
                          <span
                            className={`disposition-tag ${
                              testCase.expectations.expected_model_armor_disposition === 'BLOCK' ? 'is-block' : 'is-allow'
                            }`}
                          >
                            {testCase.expectations.expected_model_armor_disposition}
                          </span>
                        </td>
                        <td>
                          <span
                            className={`disposition-tag ${
                              testCase.expectations.expected_gateway_disposition === 'DENY' ? 'is-block' : 'is-allow'
                            }`}
                          >
                            {testCase.expectations.expected_gateway_disposition}
                          </span>
                        </td>
                        <td>
                          <span
                            className={`digest-tag ${testCase.digest_verified ? 'is-verified' : 'is-mismatch'}`}
                            title={testCase.sha256}
                          >
                            {testCase.digest_verified ? 'VERIFIED' : 'MISMATCH'}
                          </span>
                        </td>
                        <td>
                          <button
                            className="btn btn-secondary btn-inspect"
                            onClick={() => setSelectedCase(testCase)}
                          >
                            Inspect
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <div className="table-pagination">
                <span>
                  {filteredCases.length === 0
                    ? `No cases match these filters (${allCases.length} loaded)`
                    : `Showing ${corpusStart + 1}–${Math.min(corpusStart + CORPUS_PAGE_SIZE, filteredCases.length)} of ${filteredCases.length} (${allCases.length} loaded)`}
                </span>
                <div>
                  <button className="btn btn-secondary pagination-button" type="button" disabled={corpusPage === 0} onClick={() => setCorpusPage(page => Math.max(0, page - 1))} aria-label="Previous page">← Prev</button>
                  <span className="pagination-count">{corpusPage + 1} / {corpusPageCount}</span>
                  <button className="btn btn-secondary pagination-button" type="button" disabled={corpusPage >= corpusPageCount - 1} onClick={() => setCorpusPage(page => Math.min(corpusPageCount - 1, page + 1))} aria-label="Next page">Next →</button>
                </div>
              </div>
            </div>
            )}

            {/* Case Detail Modal */}
            {selectedCase && (
              <div
                className="fixed inset-0 flex items-center justify-center z-[100]"
                style={{ background: 'rgba(0,0,0,0.75)' }}
                onClick={() => setSelectedCase(null)}
                onKeyDown={event => {
                  if (event.key === 'Escape') setSelectedCase(null);
                  if (event.key !== 'Tab') return;
                  const focusable = Array.from(caseDialogRef.current?.querySelectorAll(
                    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
                  ) || []) as HTMLElement[];
                  if (!focusable?.length) return;
                  const first = focusable[0];
                  const last = focusable[focusable.length - 1];
                  if (event.shiftKey && document.activeElement === first) {
                    event.preventDefault();
                    last.focus();
                  } else if (!event.shiftKey && document.activeElement === last) {
                    event.preventDefault();
                    first.focus();
                  }
                }}
              >
                <div
                  ref={caseDialogRef}
                  className="panel w-[90%] max-w-[650px] p-8 outline-none"
                  style={{ background: 'var(--bg-secondary)' }}
                  onClick={e => e.stopPropagation()}
                  role="dialog"
                  aria-modal="true"
                  aria-labelledby="case-modal-title"
                  tabIndex={-1}
                >
                  <div className="flex justify-between items-center mb-5">
                    <h3 id="case-modal-title" className="text-[20px] font-semibold" style={{ fontFamily: 'var(--font-display)' }}>
                      Case definition: {selectedCase.case_id}
                    </h3>
                    <button
                      type="button"
                      className="p-1.5 rounded-lg transition-colors duration-150 hover:bg-[rgba(255,255,255,0.06)]"
                      style={{ background: 'transparent', border: 'none', color: 'var(--text-muted)', cursor: 'pointer' }}
                      onClick={() => setSelectedCase(null)}
                      aria-label="Close case detail"
                    >
                      <X size={16} aria-hidden="true" />
                    </button>
                  </div>

                  <div className="case-detail">
                    <p className="case-detail-description">{selectedCase.description}</p>

                    <div className="case-detail-grid">
                      <div>
                        <span>Threat family</span>
                        <strong>{selectedCase.category_name}</strong>
                        <code>{selectedCase.category}</code>
                      </div>
                      <div>
                        <span>Split</span>
                        <strong>{selectedCase.sealed ? 'Sealed holdout' : 'Development'}</strong>
                        <small>
                          {selectedCase.sealed
                            ? 'Payload withheld to keep evaluation unbiased'
                            : 'Payload published for inspection'}
                        </small>
                      </div>
                    </div>

                    <div className="case-detail-section">
                      <h4>Required behaviour</h4>
                      <p className="case-detail-note">
                        These are the assertions this case makes. They are not run results.
                      </p>
                      <dl className="case-expectations">
                        <div>
                          <dt>Outcome</dt>
                          <dd><code>{selectedCase.expectations.expected_outcome}</code></dd>
                        </div>
                        <div>
                          <dt>Model Armor</dt>
                          <dd><code>{selectedCase.expectations.expected_model_armor_disposition}</code></dd>
                        </div>
                        <div>
                          <dt>Agent Gateway</dt>
                          <dd><code>{selectedCase.expectations.expected_gateway_disposition}</code></dd>
                        </div>
                        <div>
                          <dt>Unauthorized release delta</dt>
                          <dd><code>{selectedCase.expectations.unauthorized_released_payment_delta}</code></dd>
                        </div>
                        {selectedCase.expectations.requested_tool && (
                          <div>
                            <dt>Tool the agent attempts</dt>
                            <dd><code>{selectedCase.expectations.requested_tool}</code></dd>
                          </div>
                        )}
                      </dl>
                      <div className="case-tools">
                        <div>
                          <span>Allowed</span>
                          {selectedCase.expectations.allowed_tools.map(tool => (
                            <span key={tool} className="capability-pill-success">{tool}</span>
                          ))}
                        </div>
                        <div>
                          <span>Forbidden</span>
                          {selectedCase.expectations.forbidden_tools.map(tool => (
                            <span key={tool} className="capability-pill-danger">{tool}</span>
                          ))}
                        </div>
                      </div>
                    </div>

                    <div className="case-detail-section">
                      <h4>Authored payload</h4>
                      {selectedCase.fixture ? (
                        <>
                          {typeof selectedCase.fixture.note === 'string' && (
                            <div className="case-attack-callout">
                              <span>Injected instruction</span>
                              <code>{String(selectedCase.fixture.note)}</code>
                            </div>
                          )}
                          <pre className="case-fixture">
                            {JSON.stringify(selectedCase.fixture, null, 2)}
                          </pre>
                        </>
                      ) : (
                        <p className="case-sealed-note">
                          This holdout payload is sealed. The control plane withholds it so holdout
                          results stay unbiased; only its digest and assertions are published.
                        </p>
                      )}
                    </div>

                    <div className="case-provenance">
                      <div>
                        <span>Source</span>
                        <code>{selectedCase.source_path}</code>
                      </div>
                      <div>
                        <span>Manifest digest</span>
                        <code>{selectedCase.sha256}</code>
                      </div>
                      <span className={`digest-tag ${selectedCase.digest_verified ? 'is-verified' : 'is-mismatch'}`}>
                        {selectedCase.digest_verified
                          ? 'Re-hashed by the control plane and matched'
                          : 'Digest mismatch — file differs from manifest'}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
    </>
  );
}
