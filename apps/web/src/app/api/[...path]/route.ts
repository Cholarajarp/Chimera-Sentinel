import { type NextRequest, NextResponse } from 'next/server';
import {
  healthCheck,
  getFleetPosture,
  listCandidates,
  listWorkflows,
  createWorkflow,
  getWorkflow,
  getAuditEvents,
  getEvidenceManifest,
  approveWorkflow,
  getAttestation,
  verifyAttestation,
  getCorpus,
} from '@/lib/control-plane';

/**
 * In-process control plane API router.
 *
 * Instead of proxying to a separate Rust service, this routes requests
 * directly to the control plane module running in the same Next.js process.
 * State persists within the Cloud Run container lifetime.
 */

function getTenantId(request: NextRequest): string {
  return request.headers.get('X-Tenant-ID') ?? '00000000-0000-0000-0000-000000000001';
}

function parsePathSegments(request: NextRequest): string[] {
  const url = new URL(request.url);
  // pathname is /api/v1/... — strip /api prefix
  const stripped = url.pathname.replace(/^\/api/, '');
  return stripped.split('/').filter(Boolean);
}

async function handler(request: NextRequest): Promise<NextResponse> {
  const segments = parsePathSegments(request);
  const method = request.method;
  const tenantId = getTenantId(request);

  try {
    // GET /healthz
    if (segments[0] === 'healthz' && method === 'GET') {
      return NextResponse.json(healthCheck());
    }

    // GET /v1/fleet/posture
    if (segments[0] === 'v1' && segments[1] === 'fleet' && segments[2] === 'posture' && method === 'GET') {
      return NextResponse.json(getFleetPosture(tenantId));
    }

    // GET /v1/candidates
    if (segments[0] === 'v1' && segments[1] === 'candidates' && method === 'GET') {
      const url = new URL(request.url);
      const limit = parseInt(url.searchParams.get('limit') ?? '25', 10);
      return NextResponse.json(listCandidates(tenantId, limit));
    }

    // /v1/workflows
    if (segments[0] === 'v1' && segments[1] === 'workflows') {
      // POST /v1/workflows — create workflow
      if (segments.length === 2 && method === 'POST') {
        const body = await request.json();
        return NextResponse.json(createWorkflow(tenantId, body), { status: 201 });
      }

      // GET /v1/workflows — list workflows
      if (segments.length === 2 && method === 'GET') {
        const url = new URL(request.url);
        const limit = parseInt(url.searchParams.get('limit') ?? '20', 10);
        return NextResponse.json(listWorkflows(tenantId, limit));
      }

      // Routes with workflow ID: /v1/workflows/[id]/...
      if (segments.length >= 3) {
        const workflowId = segments[2];

        // GET /v1/workflows/[id]
        if (segments.length === 3 && method === 'GET') {
          const workflow = getWorkflow(workflowId);
          if (!workflow) {
            return NextResponse.json({ error: 'Workflow not found' }, { status: 404 });
          }
          return NextResponse.json(workflow);
        }

        // GET /v1/workflows/[id]/audit
        if (segments[3] === 'audit' && method === 'GET') {
          const url = new URL(request.url);
          const limit = parseInt(url.searchParams.get('limit') ?? '100', 10);
          return NextResponse.json(getAuditEvents(workflowId, limit));
        }

        // GET /v1/workflows/[id]/evidence
        if (segments[3] === 'evidence' && method === 'GET') {
          return NextResponse.json(getEvidenceManifest(workflowId));
        }

        // POST /v1/workflows/[id]/approvals
        if (segments[3] === 'approvals' && method === 'POST') {
          const body = await request.json();
          const result = await approveWorkflow(workflowId, body);
          if (!result) {
            return NextResponse.json({ error: 'Workflow not found' }, { status: 404 });
          }
          if ('error' in result) {
            return NextResponse.json(result, { status: 409 });
          }
          return NextResponse.json(result, { status: 202 });
        }

        // GET /v1/workflows/[id]/attestation
        if (segments[3] === 'attestation' && method === 'GET') {
          return NextResponse.json(getAttestation(workflowId));
        }
      }
    }

    // POST /v1/attestations/verify
    if (segments[0] === 'v1' && segments[1] === 'attestations' && segments[2] === 'verify' && method === 'POST') {
      const body = await request.json();
      return NextResponse.json(verifyAttestation(body));
    }

    // GET /v1/corpus
    if (segments[0] === 'v1' && segments[1] === 'corpus' && method === 'GET') {
      return NextResponse.json(getCorpus(tenantId));
    }

    // 404 for unmatched routes
    return NextResponse.json(
      { error: 'Not found', path: `/${segments.join('/')}` },
      { status: 404 },
    );
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Internal server error';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}

export const GET = handler;
export const POST = handler;
export const PUT = handler;
export const DELETE = handler;
export const PATCH = handler;
