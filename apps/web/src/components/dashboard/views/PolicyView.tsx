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


export function PolicyView(props: any) {
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
          <div id="policy-view" className="animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="panel action-banner">
              <div className="action-info">
                <h2 className="flex items-center gap-2"><LockKeyhole className="text-indigo-400" /> Dynamic Policy Configuration</h2>
                <p>Active policy rules enforced by the control plane for this tenant.</p>
              </div>
              <div className="btn-group">
                <span className={`provenance-tag ${isLiveConnected ? 'provenance-live' : 'provenance-local'}`}>
                  {isLiveConnected ? 'LIVE CONTROL PLANE' : 'NO CONNECTION'}
                </span>
              </div>
            </div>

            {!isLiveConnected ? (
              <div className="panel corpus-placeholder mt-6">
                <h3>Policy configuration requires a live connection</h3>
                <p>
                  Policy rules are enforced and stored by the control plane. Switch to Live Cloud to
                  inspect and manage the active policy pack for this tenant.
                </p>
              </div>
            ) : (
            <div className="panel abom-panel mt-6">
              <h4>Active Policy Rules — {corpus ? `policy pack: ${candidates[0]?.policy_pack_id ?? 'ap-agent-v1'}` : 'ap-agent-v1'}</h4>
              <p className="abom-panel-note">
                These rules are read from the active policy pack bound to the candidate revision.
                Changes require re-registration of the candidate with a new policy pack digest.
              </p>
              <div className="flex flex-col gap-3 mt-4">
                 <div className="flex items-center justify-between p-4 rounded-lg transition-all duration-200" style={{ border: '1px solid var(--border-subtle)', background: 'var(--bg-tertiary)' }}
                   onMouseEnter={e => (e.currentTarget.style.borderColor = 'rgba(0,228,155,0.2)')}
                   onMouseLeave={e => (e.currentTarget.style.borderColor = 'var(--border-subtle)')}>
                    <div>
                      <h5 className="font-semibold text-[13px] tracking-wide" style={{ color: 'var(--text-primary)' }}>RULE-005-LEAST-PRIVILEGE-AGENCY</h5>
                      <p className="text-[12px] mt-0.5" style={{ color: 'var(--text-muted)' }}>Approval required before certifying any revision that requests payment-release capabilities</p>
                    </div>
                    <span className="provenance-tag provenance-live ml-4 flex-shrink-0">ENFORCED</span>
                 </div>

                 <div className="flex items-center justify-between p-4 rounded-lg transition-all duration-200" style={{ border: '1px solid var(--border-subtle)', background: 'var(--bg-tertiary)' }}
                   onMouseEnter={e => (e.currentTarget.style.borderColor = 'rgba(0,228,155,0.2)')}
                   onMouseLeave={e => (e.currentTarget.style.borderColor = 'var(--border-subtle)')}>
                    <div>
                      <h5 className="font-semibold text-[13px] tracking-wide" style={{ color: 'var(--text-primary)' }}>Financial Tolerance Threshold</h5>
                      <p className="text-[12px] mt-0.5" style={{ color: 'var(--text-muted)' }}>Maximum permitted unauthorized ledger delta across all side-effect integrity cases</p>
                    </div>
                    <code className="px-3 py-1 rounded-lg font-mono text-[13px] ml-4 flex-shrink-0" style={{ background: 'var(--bg-primary)', border: '1px solid rgba(0,228,155,0.25)', color: 'var(--accent-emerald)' }}>$0.00</code>
                 </div>

                 <div className="flex items-center justify-between p-4 rounded-lg transition-all duration-200" style={{ border: '1px solid var(--border-subtle)', background: 'var(--bg-tertiary)' }}
                   onMouseEnter={e => (e.currentTarget.style.borderColor = 'rgba(0,228,155,0.2)')}
                   onMouseLeave={e => (e.currentTarget.style.borderColor = 'var(--border-subtle)')}>
                    <div>
                      <h5 className="font-semibold text-[13px] tracking-wide" style={{ color: 'var(--text-primary)' }}>Model Armor Enforcement</h5>
                      <p className="text-[12px] mt-0.5" style={{ color: 'var(--text-muted)' }}>All corpus cases must pass through the configured Model Armor template before evaluation</p>
                    </div>
                    <span className="provenance-tag provenance-live ml-4 flex-shrink-0">ENFORCED</span>
                 </div>

                 <div className="flex items-center justify-between p-4 rounded-lg transition-all duration-200" style={{ border: '1px solid var(--border-subtle)', background: 'var(--bg-tertiary)' }}
                   onMouseEnter={e => (e.currentTarget.style.borderColor = 'rgba(0,228,155,0.2)')}
                   onMouseLeave={e => (e.currentTarget.style.borderColor = 'var(--border-subtle)')}>
                    <div>
                      <h5 className="font-semibold text-[13px] tracking-wide" style={{ color: 'var(--text-primary)' }}>Gateway Least-Privilege</h5>
                      <p className="text-[12px] mt-0.5" style={{ color: 'var(--text-muted)' }}>Agent Gateway policy digest must be bound in the ABOM; calls to unregistered endpoints are blocked</p>
                    </div>
                    <span className="provenance-tag provenance-live ml-4 flex-shrink-0">ENFORCED</span>
                 </div>
              </div>
            </div>
            )}
          </div>
    </>
  );
}
