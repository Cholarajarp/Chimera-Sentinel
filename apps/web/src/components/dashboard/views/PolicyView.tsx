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
            <div className="glass-panel action-banner bg-gradient-to-r from-indigo-900/20 to-slate-900">
              <div className="action-info">
                <h2 className="flex items-center gap-2"><LockKeyhole className="text-indigo-400" /> Dynamic Policy Configuration</h2>
                <p>Enterprise administrators can override strict rust-based policies and thresholds.</p>
              </div>
              <div className="btn-group">
                <button className="flex items-center gap-2 px-5 py-2.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-semibold tracking-wide shadow-lg shadow-indigo-500/20 transition-all border-0">
                   Save Policies
                </button>
              </div>
            </div>
            
            <div className="glass-panel abom-panel" style={{ marginTop: '1.5rem' }}>
              <h4>Active Policy Rules</h4>
              <div className="space-y-4 mt-4">
                 <div className="flex items-center justify-between p-5 border border-slate-700/50 rounded-xl bg-slate-800/40 backdrop-blur-sm shadow-inner transition-colors hover:border-slate-600">
                    <div>
                      <h5 className="font-semibold text-slate-200 tracking-wide text-sm">RULE-005-LEAST-PRIVILEGE-AGENCY</h5>
                      <p className="text-xs text-slate-400 mt-1">Strictly enforce approval requirement for excessive agency</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer scale-110">
                      <input type="checkbox" defaultChecked className="sr-only peer" />
                      <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500 shadow-inner"></div>
                    </label>
                 </div>

                 <div className="flex items-center justify-between p-5 border border-slate-700/50 rounded-xl bg-slate-800/40 backdrop-blur-sm shadow-inner transition-colors hover:border-slate-600">
                    <div>
                      <h5 className="font-semibold text-slate-200 tracking-wide text-sm">Financial Tolerance Threshold</h5>
                      <p className="text-xs text-slate-400 mt-1">Maximum allowed ledger deviation in tests</p>
                    </div>
                    <div className="flex items-center gap-2 bg-slate-900/80 px-3 py-1 rounded-lg border border-slate-700 focus-within:border-indigo-500 transition-colors">
                       <span className="text-slate-500 font-mono">$</span>
                       <input type="number" defaultValue="0.00" className="bg-transparent text-slate-200 text-sm w-24 text-right focus:outline-none font-mono" />
                    </div>
                 </div>
                 
                 <div className="flex items-center justify-between p-5 border border-slate-700/50 rounded-xl bg-slate-800/40 backdrop-blur-sm shadow-inner transition-colors hover:border-slate-600">
                    <div>
                      <h5 className="font-semibold text-slate-200 tracking-wide text-sm">Real-time Prompt Firewall</h5>
                      <p className="text-xs text-slate-400 mt-1">Block malicious prompt injection attempts at the gateway layer</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer scale-110">
                      <input type="checkbox" defaultChecked className="sr-only peer" />
                      <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500 shadow-inner"></div>
                    </label>
                 </div>
              </div>
            </div>
          </div>
    </>
  );
}
