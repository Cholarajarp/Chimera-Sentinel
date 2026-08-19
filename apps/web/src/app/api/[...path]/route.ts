import { type NextRequest, NextResponse } from 'next/server';

/**
 * Runtime API proxy — forwards /api/* to the Rust control plane.
 * API_URL is set at runtime via Cloud Run env var, so the URL is never
 * baked into the Docker image at build time.
 */
const API_URL = process.env.API_URL ?? 'http://localhost:8080';

async function proxy(request: NextRequest): Promise<NextResponse> {
  const { pathname, search } = new URL(request.url);

  // Strip the /api prefix — control plane routes start at /v1/ or /healthz
  const upstream = pathname.replace(/^\/api/, '');
  const target = `${API_URL}${upstream}${search}`;

  // Forward all headers except host (would confuse the upstream)
  const headers = new Headers(request.headers);
  headers.delete('host');
  // Forward Firebase ID token if present in sessionStorage-sourced header
  // (client must set X-Firebase-Token header; we just pass it through)

  const body = ['GET', 'HEAD'].includes(request.method) ? undefined : request.body;

  try {
    const upstreamResponse = await fetch(target, {
      method: request.method,
      headers,
      body,
      // Required for streaming request bodies in Node.js fetch
      // @ts-expect-error — duplex is a valid Node fetch option not yet in TS types
      duplex: 'half',
    });

    // Stream the response body back — don't buffer large payloads
    return new NextResponse(upstreamResponse.body, {
      status: upstreamResponse.status,
      statusText: upstreamResponse.statusText,
      headers: upstreamResponse.headers,
    });
  } catch (err) {
    // Control plane is unreachable (cold start, not deployed, etc.)
    const message = err instanceof Error ? err.message : 'Upstream unreachable';
    return NextResponse.json(
      { error: 'Control plane unreachable', detail: message },
      { status: 502 },
    );
  }
}

export const GET     = proxy;
export const POST    = proxy;
export const PUT     = proxy;
export const DELETE  = proxy;
export const PATCH   = proxy;
