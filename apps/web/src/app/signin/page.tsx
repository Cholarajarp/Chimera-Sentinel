'use client';

// Prevent Next.js from pre-rendering this page at build time.
// Firebase Auth requires a browser context and valid API keys at runtime.
export const dynamic = 'force-dynamic';

import Image from 'next/image';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { signInWithPopup } from 'firebase/auth';
import { ArrowLeft, ArrowRight, CheckCircle2, Github, Loader2, ShieldCheck } from 'lucide-react';
import { auth, googleProvider, githubProvider, firebaseConfigured } from '@/lib/firebase';

export default function SignInPage() {
  const router = useRouter();
  const [loading, setLoading] = useState<'google' | 'github' | null>(null);
  const [authError, setAuthError] = useState<string | null>(null);



  const handleOAuth = async (provider: 'google' | 'github') => {
    setLoading(provider);
    setAuthError(null);
    try {
      const selectedProvider = provider === 'google' ? googleProvider : githubProvider;
      const result = await signInWithPopup(auth, selectedProvider);
      const idToken = await result.user.getIdToken();
      // Store a live session cookie so middleware allows /dashboard
      const secure = window.location.protocol === 'https:' ? '; Secure' : '';
      document.cookie = `sentinel-session=live; path=/; max-age=7200; SameSite=Strict${secure}`;
      window.sessionStorage.setItem('sentinel-experience', 'live');
      // Keep the ID token in sessionStorage for optional API forwarding
      window.sessionStorage.setItem('sentinel-id-token', idToken);
      // Reviewer decisions must be attributed to the authenticated principal.
      window.sessionStorage.setItem('sentinel-reviewer', result.user.email ?? result.user.uid);
      router.push('/dashboard');
    } catch (err: unknown) {
      const code = (err as { code?: string }).code ?? '';
      if (code === 'auth/popup-closed-by-user' || code === 'auth/cancelled-popup-request') {
        // User dismissed — not an error worth showing
      } else {
        setAuthError('Sign-in failed. Please try again.');
      }
    } finally {
      setLoading(null);
    }
  };

  return (
    <main className="signin-page">
      <Link className="signin-back" href="/"><ArrowLeft size={15} /> Back to Sentinel</Link>
      <section className="signin-shell" aria-labelledby="signin-title">
        <div className="signin-brand-panel">
          <Image src="/chimera-sentinel.png" alt="Chimera Sentinel" fill priority sizes="(max-width: 760px) 100vw, 50vw" />
          <div className="signin-brand-copy">
            <span>Release admission control</span>
            <h1 id="signin-title">Enter the evidence workspace.</h1>
            <p>Inspect one exact agent revision from adversarial evaluation through constrained approval and cryptographic attestation.</p>
          </div>
        </div>
        <div className="signin-options">
          <div className="signin-heading">
            <ShieldCheck size={22} aria-hidden="true" />
            <div><span>Chimera Sentinel</span><h2>Choose access mode</h2></div>
          </div>

          {authError && (
            <div style={{ marginBottom: '0.75rem', padding: '0.6rem 0.85rem', background: 'rgba(244,63,94,0.12)', border: '1px solid rgba(244,63,94,0.3)', borderRadius: '6px', color: '#fda4af', fontSize: '0.78rem' }} role="alert">
              {authError}
            </div>
          )}

          {firebaseConfigured || process.env.NODE_ENV === 'development' ? (
            <>
              <button className="signin-option" type="button" onClick={() => handleOAuth('google')} disabled={loading !== null}>
                <span className="signin-option-icon">{loading === 'google' ? <Loader2 size={19} className="spinner" /> : <GoogleMark />}</span>
                <span><strong>Continue with Google</strong><small>Organization workspace · Live Cloud enabled</small></span>
                <ArrowRight size={16} />
              </button>
              <button className="signin-option" type="button" onClick={() => handleOAuth('github')} disabled={loading !== null}>
                <span className="signin-option-icon">{loading === 'github' ? <Loader2 size={19} className="spinner" /> : <Github size={19} />}</span>
                <span><strong>Continue with GitHub</strong><small>Organization workspace · Live Cloud enabled</small></span>
                <ArrowRight size={16} />
              </button>
            </>
          ) : (
            <>
              <button className="signin-option" type="button" disabled title="Firebase not configured for this deployment">
                <span className="signin-option-icon"><GoogleMark /></span>
                <span><strong>Continue with Google</strong><small>Configure NEXT_PUBLIC_FIREBASE_* to enable</small></span>
                <ArrowRight size={16} />
              </button>
              <button className="signin-option" type="button" disabled title="Firebase not configured for this deployment">
                <span className="signin-option-icon"><Github size={19} /></span>
                <span><strong>Continue with GitHub</strong><small>Configure NEXT_PUBLIC_FIREBASE_* to enable</small></span>
                <ArrowRight size={16} />
              </button>
            </>
          )}

          <p className="signin-policy">Sentinel verifies API health on connection and rejects unverifiable data.</p>
        </div>
      </section>
    </main>
  );
}

function GoogleMark() {
  return <svg viewBox="0 0 24 24" width="19" height="19" aria-hidden="true"><path fill="#4285f4" d="M21.6 12.23c0-.71-.06-1.4-.18-2.05H12v3.88h5.38a4.6 4.6 0 0 1-2 3.02v2.52h3.24c1.9-1.75 2.98-4.32 2.98-7.37Z"/><path fill="#34a853" d="M12 22c2.7 0 4.96-.9 6.62-2.4l-3.24-2.52c-.9.6-2.05.96-3.38.96-2.6 0-4.8-1.76-5.6-4.13H3.06v2.6A10 10 0 0 0 12 22Z"/><path fill="#fbbc05" d="M6.4 13.91A6 6 0 0 1 6.08 12c0-.66.11-1.3.32-1.91v-2.6H3.06A10 10 0 0 0 2 12c0 1.61.39 3.14 1.06 4.51l3.34-2.6Z"/><path fill="#ea4335" d="M12 5.96c1.47 0 2.78.5 3.82 1.49l2.87-2.87A9.6 9.6 0 0 0 12 2a10 10 0 0 0-8.94 5.49l3.34 2.6A5.96 5.96 0 0 1 12 5.96Z"/></svg>;
}
