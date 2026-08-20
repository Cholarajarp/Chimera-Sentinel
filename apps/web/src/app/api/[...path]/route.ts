import { type NextRequest, NextResponse } from 'next/server';

/**
 * Sentinel API gateway — strict pass-through boundary.
 *
 * Every request is forwarded verbatim to the Rust control plane (Cloud Run).
 * The Rust service owns all durable state (Firestore), ADK evaluation
 * (via the Python certifier), and Cloud KMS signing. The Next.js process is a
 * pure proxy: it never holds canonical state and has no simulation fallback.
 *
 * Configuration: the Terraform output `control_plane_url` is set as the
 * `API_URL` environment variable on the Cloud Run web service (see
 * infra/terraform/main.tf `API_URL = google_cloud_run_v2_service.control_plane.uri`).
 *
 * If `API_URL` is missing, the proxy refuses to serve rather than silently
 * falling back to a mock: it returns a fatal 500 so the misconfiguration is
 * visible and never confused with a live control-plane response.
 */

// Evaluated per request (never cached at module load) so config changes are
// picked up. Server-side only — never exposed to the browser.
function getUpstreamUrl(): string {
  return (process.env.API_URL ?? '').replace(/\/$/, '');
}

/**
 * Forward the incoming Next.js request verbatim to the Rust control plane,
 * stripping the /api prefix so the upstream receives /healthz, /v1/…, etc.
 *
 * Cloud Run service-to-service auth uses OIDC identity tokens obtained from
 * the metadata server. The token audience is the upstream Cloud Run URL.
 */
async function proxyToRust(upstreamUrl: string, request: NextRequest): Promise<NextResponse> {
  const body = request.method !== 'GET' && request.method !== 'HEAD'
    ? await request.arrayBuffer()
    : undefined;

  const url = new URL(request.url);

  // Strip the /api prefix — upstream expects /healthz, /v1/...
  const upstreamPath = url.pathname.replace(/^\/api/, '') || '/';
  const upstreamFullUrl = `${upstreamUrl}${upstreamPath}${url.search}`;

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
      `http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity?audience=${encodeURIComponent(upstreamUrl)}&format=full`,
      { headers: { 'Metadata-Flavor': 'Google' }, signal: AbortSignal.timeout(2_000) },
    );
    if (tokenResp.ok) {
      forwardHeaders.set('Authorization', `Bearer ${await tokenResp.text()}`);
    }
  } catch {
    // Not on GCP — identity token unavailable; proceed without it.
  }

  const upstream = await fetch(upstreamFullUrl, {
    method: request.method,
    headers: forwardHeaders,
    body: body ? Buffer.from(body) : undefined,
    // @ts-expect-error Node 18 fetch supports duplex for streaming bodies
    duplex: body ? 'half' : undefined,
    signal: AbortSignal.timeout(60_000),
    cache: 'no-store',
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

async function handler(request: NextRequest): Promise<NextResponse> {
  const upstreamUrl = getUpstreamUrl();

  if (!upstreamUrl) {
    // Fatal misconfiguration: refuse to serve rather than fall back to a mock.
    // Throwing inside a Next.js route handler surfaces as a 500 response.
    throw new Error(
      'Sentinel API gateway misconfigured: API_URL env var is not set. ' +
        'The web console is a strict proxy and has no simulation fallback. ' +
        'Set API_URL to the Rust control plane URL (Terraform output `control_plane_url`).',
    );
  }

  return proxyToRust(upstreamUrl, request);
}

export const GET = handler;
export const POST = handler;
export const PUT = handler;
export const DELETE = handler;
export const PATCH = handler;
