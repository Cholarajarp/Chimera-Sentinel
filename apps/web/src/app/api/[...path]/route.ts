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
 * Sentinel API gateway.
 *
 * Production path (API_URL is set):
 *   Every request is forwarded to the Rust control plane running on Cloud Run.
 *   The Rust service owns all durable state (Firestore), ADK evaluation
 *   (via the Python certifier), and Cloud KMS signing.  The Next.js process
 *   is a pure pass-through; it never touches the data.
 *
 * Local-dev / cold-start fallback (API_URL is absent):
 *   Requests are handled in-process by control-plane.ts.  This path exists
 *   only for running the UI without standing up the full Rust stack locally.
 *   It uses in-memory Maps and a SHA-256 stand-in instead of real KMS —
 *   that is intentional and clearly labelled in every response.
 *
 * The Terraform output `control_plane_url` must be set as the API_URL
 * environment variable on the Cloud Run web service (it already is — see
 * infra/terraform/main.tf `API_URL = google_cloud_run_v2_service.control_plane.uri`).
 */

// Server-side only — never exposed to the browser.
const UPSTREAM_URL = process.env.API_URL?.replace(/\/$/, '') ?? '';

function isProxyMode(): boolean {
  return UPSTREAM_URL.length > 0;
}

/**
 * Forward the incoming Next.js request verbatim to the Rust control plane,
 * stripping the /api prefix so the upstream receives /healthz, /v1/…, etc.
 *
 * Cloud Run service-to-service auth uses OIDC identity tokens obtained from
 * the metadata server.  The token audience is the upstream Cloud Run URL.
 */
async function proxyToRust(request: NextRequest): Promise<NextResponse> {
  const url = new URL(request.url);

  // Strip the /api prefix — upstream expects /healthz, /v1/...
  const upstreamPath = url.pathname.replace(/^\/api/, '') || '/';
  const upstreamUrl = `${UPSTREAM_URL}${upstreamPath}${url.search}`;

  // Build forwarded headers: preserve X-Tenant-ID, Content-Type, etc.
  const forwardHeaders = new Headers();
  for (const [key, value] of request.headers.entries()) {
    // Drop hop-by-hop and host headers before forwarding
    if (/^(host|connection|transfer-encoding|keep-alive|te|trailers|upgrade)$/i.test(key)) {
      continue;
    }
    forwardHeaders.set(key, value);
  }

  // Attach a Cloud Run OIDC identity token when running on GCP.
  // On Cloud Run the metadata server is always reachable; on local dev we skip
  // this header and rely on the Rust service's open IAM policy or dev mode.
  try {
    const tokenResp = await fetch(
      `http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity?audience=${encodeURIComponent(UPSTREAM_URL)}&format=full`,
      { headers: { 'Metadata-Flavor': 'Google' }, signal: AbortSignal.timeout(2_000) },
    );
    if (tokenResp.ok) {
      forwardHeaders.set('Authorization', `Bearer ${await tokenResp.text()}`);
    }
  } catch {
    // Not on GCP — identity token unavailable; proceed without it.
  }

  const body = request.method !== 'GET' && request.method !== 'HEAD'
    ? await request.arrayBuffer()
    : undefined;

  const upstream = await fetch(upstreamUrl, {
    method: request.method,
    headers: forwardHeaders,
    body: body ? Buffer.from(body) : undefined,
    // @ts-expect-error Node 18 fetch supports duplex for streaming bodies
    duplex: body ? 'half' : undefined,
    signal: AbortSignal.timeout(60_000),
  });

  // Stream the response back to the browser, preserving status and headers.
  const responseHeaders = new Headers();
  for (const [key, value] of upstream.headers.entries()) {
    if (/^(transfer-encoding|connection)$/i.test(key)) continue;
    responseHeaders.set(key, value);
  }

  return new NextResponse(upstream.body, {
    status: upstream.status,
    statusText: upstream.statusText,
    headers: responseHeaders,
  });
}

// ── In-process fallback (local dev / no API_URL) ─────────────────────────────

function getTenantId(request: NextRequest): string {
  return request.headers.get('X-Tenant-ID') ?? '00000000-0000-0000-0000-000000000001';
}

function parsePathSegments(request: NextRequest): string[] {
  const url = new URL(request.url);
  const stripped = url.pathname.replace(/^\/api/, '');
  return stripped.split('/').filter(Boolean);
}

async function inProcessHandler(request: NextRequest): Promise<NextResponse> {
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
      return NextResponse.json(await getFleetPosture(tenantId));
    }

    // GET /v1/candidates
    if (segments[0] === 'v1' && segments[1] === 'candidates' && method === 'GET') {
      const url = new URL(request.url);
      const limit = parseInt(url.searchParams.get('limit') ?? '25', 10);
      return NextResponse.json(await listCandidates(tenantId, limit));
    }

    // /v1/workflows
    if (segments[0] === 'v1' && segments[1] === 'workflows') {
      if (segments.length === 2 && method === 'POST') {
        const body = await request.json();
        return NextResponse.json(await createWorkflow(tenantId, body), { status: 201 });
      }

      if (segments.length === 2 && method === 'GET') {
        const url = new URL(request.url);
        const limit = parseInt(url.searchParams.get('limit') ?? '20', 10);
        return NextResponse.json(await listWorkflows(tenantId, limit));
      }

      if (segments.length >= 3) {
        const workflowId = segments[2];

        if (segments.length === 3 && method === 'GET') {
          const workflow = await getWorkflow(workflowId);
          if (!workflow) {
            return NextResponse.json({ error: 'Workflow not found' }, { status: 404 });
          }
          return NextResponse.json(workflow);
        }

        if (segments[3] === 'audit' && method === 'GET') {
          const url = new URL(request.url);
          const limit = parseInt(url.searchParams.get('limit') ?? '100', 10);
          return NextResponse.json(await getAuditEvents(workflowId, limit));
        }

        if (segments[3] === 'evidence' && method === 'GET') {
          return NextResponse.json(await getEvidenceManifest(workflowId));
        }

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

        if (segments[3] === 'attestation' && method === 'GET') {
          return NextResponse.json(await getAttestation(workflowId));
        }
      }
    }

    // POST /v1/attestations/verify
    if (segments[0] === 'v1' && segments[1] === 'attestations' && segments[2] === 'verify' && method === 'POST') {
      const body = await request.json();
      return NextResponse.json(await verifyAttestation(body));
    }

    // GET /v1/corpus
    if (segments[0] === 'v1' && segments[1] === 'corpus' && method === 'GET') {
      return NextResponse.json(await getCorpus(tenantId));
    }

    return NextResponse.json(
      { error: 'Not found', path: `/${segments.join('/')}` },
      { status: 404 },
    );
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Internal server error';
    return NextResponse.json({ error: message }, { status: 500 });
  }
}

// ── Unified handler ───────────────────────────────────────────────────────────

async function handler(request: NextRequest): Promise<NextResponse> {
  if (isProxyMode()) {
    return proxyToRust(request);
  }
  return inProcessHandler(request);
}

export const GET = handler;
export const POST = handler;
export const PUT = handler;
export const DELETE = handler;
export const PATCH = handler;
