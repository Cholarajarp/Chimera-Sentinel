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


export function EvidenceView(props: any) {
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
          <div id="evidence-view">
            <div className="panel action-banner">
              <div className="action-info">
                <h2>Content-Addressed Evidence Bundle Manifest</h2>
                <p>RFC 8785 JSON Canonicalization Scheme (JCS) verified objects</p>
              </div>
              <div className="btn-group">
                {liveManifest && (
                  <span className="provenance-tag provenance-live" title={liveManifest.manifest_digest}>
                    {shortDigest(liveManifest.manifest_digest)}
                  </span>
                )}
              </div>
            </div>

            <div className="panel p-6 mb-6">
              <div className="table-wrapper">
                <table className="sentinel-table">
                  <thead>
                    <tr>
                      <th>Object Path</th>
                      <th>Schema Version</th>
                      <th>SHA-256 Digest</th>
                      <th>Size</th>
                      <th>Producer</th>
                      <th>Provenance</th>
                    </tr>
                  </thead>
                  <tbody>
                    {liveManifest ? liveManifest.objects.map(object => (
                      <tr key={object.path}>
                        <td><code>{object.path}</code></td>
                        <td>{object.schema_version}</td>
                        <td className="font-mono text-[12.5px]">{object.sha256_digest}</td>
                        <td>{(object.size_bytes / 1024).toFixed(1)} KB</td>
                        <td>{object.producer}</td>
                        <td><span className="provenance-tag provenance-live">{object.provenance}</span></td>
                      </tr>
                    )) : (
                      <tr>
                        <td colSpan={6}>
                          {isLiveConnected
                            ? 'No evidence manifest has been produced for this workflow yet. Objects appear once a run records them.'
                            : 'Evidence objects are content-addressed artifacts written by the control plane. Switch to Live Cloud and complete a run to load a real manifest.'}
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
    </>
  );
}
