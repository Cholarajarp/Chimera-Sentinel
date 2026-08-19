import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

/**
 * Protects /dashboard from direct navigation.
 * Accepts only:
 *   sentinel-session=live  — set by Firebase Google/GitHub sign-in
 * Any other value or missing cookie redirects to /signin.
 *
 * /signin itself is always allowed through (prevents a redirect loop).
 */
export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Never intercept the sign-in page itself or static assets
  if (pathname.startsWith('/signin') || pathname.startsWith('/_next') || pathname.startsWith('/api')) {
    return NextResponse.next();
  }

  const session = request.cookies.get('sentinel-session')?.value;
  const isValid = session === 'live';

  if (!isValid) {
    const signinUrl = new URL('/signin', request.url);
    // Preserve intended destination so sign-in can redirect back
    signinUrl.searchParams.set('next', pathname);
    return NextResponse.redirect(signinUrl);
  }

  return NextResponse.next();
}

export const config = {
  // Only run on dashboard routes — not on signin, api, or static assets
  matcher: ['/dashboard/:path*'],
};
