import React, { useEffect, useState } from 'react';
import { useDashboard } from '../DashboardContext';
import { LockKeyhole, Activity, AlertTriangle } from 'lucide-react';

const API_BASE = '/api';
const TENANT_ID = '00000000-0000-0000-0000-000000000001';

interface PolicyRule {
  id: string;
  name: string;
  description: string;
  severity: string;
  rule_type: string;
}

interface PolicyPack {
  schema_version: string;
  id: string;
  version: string;
  name: string;
  description: string;
  digest: string;
  required_managed_controls: string[];
  rules: PolicyRule[];
  provenance: string;
}

export function PolicyView(props: any) {
  const { isLiveConnected, apiReachable, candidates } = useDashboard();

  const [policy, setPolicy] = useState<PolicyPack | null>(null);
  const [policyStatus, setPolicyStatus] = useState<'loading' | 'ready' | 'error'>('loading');
  const [policyError, setPolicyError] = useState<string | null>(null);

  useEffect(() => {
    if (!isLiveConnected) return;
    setPolicyStatus('loading');
    setPolicyError(null);
    fetch(`${API_BASE}/v1/policy`, { headers: { 'X-Tenant-ID': TENANT_ID } })
      .then(async r => {
        if (!r.ok) throw new Error(`Policy fetch failed: ${r.status}`);
        setPolicy(await r.json() as PolicyPack);
        setPolicyStatus('ready');
      })
      .catch((e: unknown) => {
        setPolicyError(e instanceof Error ? e.message : String(e));
        setPolicyStatus('error');
      });
  }, [isLiveConnected]);

  const severityClass = (s: string) => {
    if (s === 'CRITICAL') return 'provenance-tag' + ' ' + 'is-critical';
    if (s === 'HIGH') return 'risk-tag is-high';
    return 'risk-tag is-low';
  };

  return (
    <>
      <div id="policy-view" className="animate-in fade-in slide-in-from-bottom-4 duration-500">
        <div className="panel action-banner">
          <div className="action-info">
            <h2 className="flex items-center gap-2"><LockKeyhole className="text-indigo-400" /> Active Policy Pack</h2>
            <p>
              {policy
                ? `${policy.name} · v${policy.version}`
                : 'Policy rules loaded from the control plane for this tenant.'}
            </p>
          </div>
          <div className="btn-group">
            {policy && (
              <span className="provenance-tag provenance-live" title={policy.digest}>
                {policy.id} · v{policy.version}
              </span>
            )}
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
              inspect the active policy pack for this tenant.
            </p>
          </div>
        ) : policyStatus === 'loading' ? (
          <div className="panel corpus-placeholder mt-6" role="status">
            <Activity size={18} className="spinner" aria-hidden="true" />
            <p>Loading policy pack from the control plane…</p>
          </div>
        ) : policyStatus === 'error' ? (
          <div className="panel corpus-placeholder is-error mt-6" role="alert">
            <h3>Policy pack unavailable</h3>
            <p>{policyError}</p>
            <p className="corpus-placeholder-note">
              Deploy with <code>policy-packs/ap-agent-v1/pack.json</code> accessible at{' '}
              <code>SENTINEL_POLICY_PACK</code>.
            </p>
          </div>
        ) : policy ? (
          <>
            <div className="panel abom-panel mt-6">
              <h4>Policy pack metadata</h4>
              <div className="abom-fields">
                <div><span>Pack ID</span><code>{policy.id}</code></div>
                <div><span>Version</span><code>{policy.version}</code></div>
                <div><span>Pack digest</span><code className="is-digest">{policy.digest}</code></div>
                <div>
                  <span>Required managed controls</span>
                  <div className="abom-capabilities mt-1">
                    {policy.required_managed_controls.map(c => (
                      <span key={c} className="capability-pill-success">{c}</span>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            <div className="panel abom-panel mt-6">
              <h4>Enforced rules ({policy.rules.length})</h4>
              <p className="abom-panel-note">
                These rules are read directly from the policy pack on disk. Changes require
                re-registration of the candidate with a new policy pack digest.
              </p>
              <div className="flex flex-col gap-3 mt-4">
                {policy.rules.map(rule => (
                  <div
                    key={rule.id}
                    className="flex items-start justify-between p-4 rounded-lg"
                    style={{ border: '1px solid var(--border-subtle)', background: 'var(--bg-tertiary)' }}
                  >
                    <div>
                      <h5 className="font-semibold text-[13px] tracking-wide" style={{ color: 'var(--text-primary)' }}>{rule.id}</h5>
                      <p className="text-[12.5px] font-medium mt-0.5" style={{ color: 'var(--text-accent)' }}>{rule.name}</p>
                      <p className="text-[12px] mt-1" style={{ color: 'var(--text-muted)' }}>{rule.description}</p>
                      <code className="text-[11px] mt-1 inline-block" style={{ color: 'var(--text-muted)' }}>{rule.rule_type}</code>
                    </div>
                    <div className="flex flex-col items-end gap-1 ml-4 flex-shrink-0">
                      <span className="provenance-tag provenance-live">ENFORCED</span>
                      <span className={`risk-tag ${rule.severity === 'CRITICAL' ? 'is-high' : 'is-medium'}`}>
                        {rule.severity}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </>
        ) : null}
      </div>
    </>
  );
}
