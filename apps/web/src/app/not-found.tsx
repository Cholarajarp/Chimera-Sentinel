import Link from 'next/link';
import { ShieldCheck } from 'lucide-react';

export default function NotFound() {
  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'var(--bg-primary)',
        color: 'var(--text-primary)',
        padding: '2rem',
        gap: '1.5rem',
        textAlign: 'center',
      }}
    >
      <ShieldCheck size={48} style={{ color: 'var(--accent-emerald)', opacity: 0.7 }} aria-hidden="true" />
      <div>
        <p
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '0.72rem',
            fontWeight: 600,
            textTransform: 'uppercase',
            letterSpacing: '0.1em',
            color: 'var(--text-muted)',
            marginBottom: '0.75rem',
          }}
        >
          404 — Route not found
        </p>
        <h1
          style={{
            fontFamily: 'var(--font-display)',
            fontSize: 'clamp(2rem, 5vw, 3.5rem)',
            fontWeight: 700,
            lineHeight: 1.05,
            marginBottom: '1rem',
          }}
        >
          This page does not exist.
        </h1>
        <p style={{ color: 'var(--text-secondary)', fontSize: '1rem', maxWidth: '480px', margin: '0 auto 1.5rem' }}>
          The evidence workspace you are looking for was not found. Return to the landing page or open mission control.
        </p>
      </div>
      <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap', justifyContent: 'center' }}>
        <Link
          href="/"
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            minHeight: '44px',
            padding: '0 1.25rem',
            borderRadius: '8px',
            background: 'var(--text-primary)',
            color: 'var(--bg-primary)',
            fontWeight: 700,
            fontSize: '0.875rem',
            textDecoration: 'none',
          }}
        >
          Back to Sentinel
        </Link>
        <Link
          href="/signin"
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            minHeight: '44px',
            padding: '0 1.25rem',
            borderRadius: '8px',
            border: '1px solid var(--border-subtle)',
            background: 'var(--bg-tertiary)',
            color: 'var(--text-primary)',
            fontWeight: 700,
            fontSize: '0.875rem',
            textDecoration: 'none',
          }}
        >
          Open mission control
        </Link>
      </div>
    </div>
  );
}
