import React, { useState } from 'react';
import { X, UploadCloud, AlertCircle, Loader2 } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface SubmitCandidateModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

const DEFAULT_ABOM = `{
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "agent_id": "demo-agent-v1",
  "policy_pack_id": "ap-agent-v1",
  "corpus_version": "v1.0.0",
  "abom": {
    "source_digest": "sha256:aabbccdd11223344556677889900aabbccdd11223344556677889900aabbccdd",
    "registry_resource": "projects/chimera-sentinel/locations/us-east1/agents/demo-agent",
    "runtime_resource": "projects/chimera-sentinel/locations/us-east1/agents/demo-agent/runtimes/default",
    "agent_identity": "sa-demo@chimera-sentinel.iam.gserviceaccount.com",
    "model_ref": "vertex-ai:gemini-3.5-flash@2026-08-01",
    "prompt_config_digest": "sha256:001122334455667788990011223344556677889900112233445566778899001122",
    "tool_manifest_digest": "sha256:334455667788990011223344556677889900112233445566778899001122334455",
    "memory_config_digest": "sha256:556677889900112233445566778899001122334455667788990011223344556677",
    "gateway_policy_digest": "sha256:667788990011223344556677889900112233445566778899001122334455667788",
    "model_armor_config_digest": "sha256:99aabb001122334455667788990011223344556677889900112233445566778899",
    "requested_capabilities": ["draft_invoice_payment", "get_payment_status", "lookup_vendor"],
    "data_classification": ["CONFIDENTIAL", "PII"],
    "environment": "production",
    "owner": "engineer@chimera-sentinel.iam.gserviceaccount.com",
    "provenance": "LOCAL",
    "recorded_at": "2026-08-20T10:00:00Z",
    "risk_tier": "HIGH"
  }
}`;

export const SubmitCandidateModal: React.FC<SubmitCandidateModalProps> = ({ isOpen, onClose, onSuccess }) => {
  const [abomJson, setAbomJson] = useState(DEFAULT_ABOM);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsLoading(true);

    try {
      let parsed;
      try {
        parsed = JSON.parse(abomJson);
      } catch (e) {
        throw new Error('Invalid JSON format. Please check your syntax.');
      }
      
      const res = await fetch('/api/v1/candidates', {
        method: 'POST',
        headers: { 
          'Content-Type': 'application/json',
          'X-Tenant-ID': parsed.tenant_id || '00000000-0000-0000-0000-000000000001'
        },
        body: JSON.stringify(parsed)
      });

      if (!res.ok) {
        const text = await res.text();
        throw new Error(text || `Status ${res.status}`);
      }

      onSuccess();
      onClose();
    } catch (err: any) {
      setError(err.message || 'Failed to submit ABOM');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80">
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            className="panel w-full max-w-2xl overflow-hidden flex flex-col shadow-2xl"
            style={{ padding: 0 }}
          >
            <div className="flex items-center justify-between p-5" style={{ borderBottom: '1px solid var(--border-subtle)' }}>
              <h2 className="text-xl font-semibold flex items-center gap-2" style={{ color: 'var(--text-primary)' }}>
                <UploadCloud className="w-5 h-5 text-emerald-500" />
                Register Agent Candidate
              </h2>
              <button
                onClick={onClose}
                className="p-1.5 transition-colors"
                style={{ color: 'var(--text-secondary)', background: 'transparent', border: 'none', cursor: 'pointer' }}
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            
            <form onSubmit={handleSubmit} className="flex flex-col p-5 gap-4">
              <div className="flex flex-col gap-2">
                <label className="text-sm font-medium" style={{ color: 'var(--text-primary)' }}>
                  Agent Bill of Materials (JSON)
                </label>
                <p className="text-xs" style={{ color: 'var(--text-secondary)' }}>
                  Paste your compiled ABOM manifest to register the agent into the Sentinel platform.
                </p>
                <textarea
                  value={abomJson}
                  onChange={(e) => setAbomJson(e.target.value)}
                  className="w-full h-80 p-4 font-mono text-sm rounded-sm focus:outline-none transition-colors resize-none custom-scrollbar"
                  style={{ 
                    background: 'var(--bg-secondary)', 
                    border: '1px solid var(--border-mid)', 
                    color: 'var(--text-primary)',
                    boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.02)'
                  }}
                  spellCheck={false}
                />
              </div>

              {error && (
                <div className="flex items-center gap-2 p-3 text-sm rounded-sm" style={{ background: 'var(--accent-rose-dim)', border: '1px solid rgba(251, 75, 110, 0.3)', color: '#fb4b6e' }}>
                  <AlertCircle className="w-4 h-4 shrink-0" />
                  <span className="break-all">{error}</span>
                </div>
              )}

              <div className="flex justify-end pt-4 gap-3" style={{ borderTop: '1px solid var(--border-subtle)' }}>
                <button
                  type="button"
                  onClick={onClose}
                  className="btn"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={isLoading}
                  className="btn btn-primary"
                >
                  {isLoading ? <Loader2 className="w-4 h-4 mr-2 animate-spin" /> : <UploadCloud className="w-4 h-4 mr-2" />}
                  Submit Candidate
                </button>
              </div>
            </form>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
};
