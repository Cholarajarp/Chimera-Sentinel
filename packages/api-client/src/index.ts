/**
 * Chimera Sentinel TypeScript Client SDK
 */

export interface CandidateRevisionRequest {
  tenant_id: string;
  agent_id: string;
  abom: AgentBillOfMaterials;
  policy_pack_id: string;
  corpus_version: string;
}

export interface AgentBillOfMaterials {
  source_digest: string;
  registry_resource: string;
  runtime_resource: string;
  agent_identity: string;
  model_ref: string;
  prompt_config_digest: string;
  tool_manifest_digest: string;
  memory_config_digest: string;
  gateway_policy_digest: string;
  model_armor_config_digest: string;
  requested_capabilities: string[];
  data_classification: string[];
  environment: string;
  owner: string;
  risk_tier: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  provenance: 'LIVE' | 'REPLAY' | 'DEMO' | 'INFERRED' | 'LOCAL';
  recorded_at: string;
}

export interface CandidateRevisionResponse {
  tenant_id: string;
  agent_id: string;
  revision_id: string;
  created_at: string;
}

export interface CreateCertificationRequest {
  tenant_id: string;
  candidate_revision_id: string;
  policy_pack_id: string;
  corpus_version: string;
  idempotency_key: string;
}

export interface CreateCertificationResponse {
  workflow_id: string;
  state: string;
  created_at: string;
}

export interface WorkflowResponse {
  workflow_id: string;
  tenant_id: string;
  candidate_revision_id: string;
  state: string;
  version: number;
  policy_pack_id: string;
  corpus_version: string;
  gate_decision?: string;
  gate_explanation?: string;
  approval_id?: string;
  attestation_digest?: string;
  created_at: string;
  updated_at: string;
}

export interface SubmitApprovalRequest {
  tenant_id: string;
  reviewer: string;
  reviewer_role: string;
  proposed_capabilities: string[];
  approved_capabilities: string[];
  reason_code: string;
  note?: string;
  duration_seconds: number;
}

export class SentinelClient {
  constructor(private baseUrl: string = 'http://localhost:8080', private tenantId?: string) {}

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(this.tenantId ? { 'X-Tenant-ID': this.tenantId } : {}),
      ...(options.headers as Record<string, string>),
    };

    const res = await fetch(`${this.baseUrl}${path}`, {
      ...options,
      headers,
    });

    if (!res.ok) {
      throw new Error(`Sentinel API Error ${res.status}: ${await res.text()}`);
    }

    return res.json();
  }

  async createCandidate(req: CandidateRevisionRequest): Promise<CandidateRevisionResponse> {
    return this.request('/v1/candidates', {
      method: 'POST',
      body: JSON.stringify(req),
    });
  }

  async startCertification(req: CreateCertificationRequest): Promise<CreateCertificationResponse> {
    return this.request('/v1/workflows', {
      method: 'POST',
      body: JSON.stringify(req),
    });
  }

  async getWorkflow(workflowId: string): Promise<WorkflowResponse> {
    return this.request(`/v1/workflows/${workflowId}`);
  }

  async submitApproval(workflowId: string, req: SubmitApprovalRequest): Promise<any> {
    return this.request(`/v1/workflows/${workflowId}/approvals`, {
      method: 'POST',
      body: JSON.stringify(req),
    });
  }

  async verifyAttestation(attestation: any, environment = 'production'): Promise<any> {
    return this.request('/v1/attestations/verify', {
      method: 'POST',
      body: JSON.stringify({
        tenant_id: this.tenantId || '00000000-0000-0000-0000-000000000001',
        attestation,
        environment,
      }),
    });
  }
}
