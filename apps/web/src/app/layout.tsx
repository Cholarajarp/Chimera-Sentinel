import './globals.css';
import type { Metadata } from 'next';
import { ThemeProvider } from '@/components/ThemeProvider';

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL ?? 'https://chimera-sentinel.web.app';

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: 'Chimera Sentinel | Release Admission & Continuous Assurance for Enterprise AI Agents',
  description:
    'Fortified release admission controller for AI agents on Google Cloud. Cryptographically verifiable attestations, deterministic ledger side-effect oracles, Model Armor, and Agent Gateway least-privilege governance.',
  openGraph: {
    title: 'Chimera Sentinel',
    description: 'Make every AI-agent revision earn production authority through adversarial evidence and a signed attestation.',
    images: ['/chimera-sentinel.png'],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'Chimera Sentinel',
    description: 'Production admission control and continuous assurance for enterprise AI agents.',
    images: ['/chimera-sentinel.png'],
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600&family=Outfit:wght@400;500;600;700;800&display=swap"
          rel="stylesheet"
        />
      </head>
      <body>
        <ThemeProvider>
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
