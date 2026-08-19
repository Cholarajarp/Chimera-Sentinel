import Image from 'next/image';
import Link from 'next/link';
import {
  ArrowRight,
  Check,
  Cloud,
  Database,
  ExternalLink,
  FileKey2,
  Fingerprint,
  LockKeyhole,
  Play,
  ShieldCheck,
} from 'lucide-react';

const controls = [
  {
    icon: Fingerprint,
    title: 'Bind the exact revision',
    body: 'Model, prompts, tools, identity, and policy become one immutable Agent Bill of Materials.',
  },
  {
    icon: LockKeyhole,
    title: 'Attack before release',
    body: 'Eighty safe and hostile workflows exercise Model Armor, Agent Gateway, and agent identity boundaries.',
  },
  {
    icon: Database,
    title: 'Verify real side effects',
    body: 'A deterministic ERP ledger oracle proves that unauthorized payment release remains exactly zero.',
  },
  {
    icon: FileKey2,
    title: 'Issue production authority',
    body: 'Only a constrained revision earns an expiring, offline-verifiable Cloud KMS attestation.',
  },
];

const stages = ['Register revision', 'Run adversarial corpus', 'Verify ledger', 'Human constraint', 'Retest', 'KMS attest'];

export default function LandingPage() {
  return (
    <div className="landing-page">
      <header className="landing-nav">
        <Link href="/" className="landing-brand" aria-label="Chimera Sentinel home">
          <Image src="/chimera-sentinel.png" alt="" width={38} height={38} priority />
          <span>Chimera Sentinel</span>
        </Link>
        <nav aria-label="Primary navigation">
          <a href="#how-it-works">How it works</a>
          <a href="#architecture">Architecture</a>
          <a href="https://github.com/Cholarajarp" target="_blank" rel="noreferrer">
            Source <ExternalLink size={13} aria-hidden="true" />
          </a>
          <Link href="/signin" className="landing-nav-cta">
            Open mission control <ArrowRight size={15} aria-hidden="true" />
          </Link>
        </nav>
      </header>

      <main>
        <section className="landing-hero">
          <Image
            src="/chimera-sentinel.png"
            alt="Chimera Sentinel, represented by a three-part cryptographic guardian"
            fill
            priority
            sizes="100vw"
            className="landing-hero-art"
          />
          <div className="landing-hero-scrim" />
          <div className="landing-hero-content">
            <div className="landing-kicker">
              <Cloud size={15} aria-hidden="true" />
              Google Cloud · IBM Bob Track
            </div>
            <h1>Production authority must be earned.</h1>
            <p>
              Chimera Sentinel attacks an exact AI-agent revision, verifies its real business side effects,
              removes excessive authority, and signs the evidence required for production admission.
            </p>
            <div className="landing-hero-actions">
              <Link href="/signin" className="landing-primary-action">
                <Play size={17} aria-hidden="true" />
                Open Mission Control
              </Link>
              <a href="#how-it-works" className="landing-secondary-action">
                See the control loop
              </a>
            </div>
            <div className="landing-proof-line" aria-label="Core proof points">
              <span><strong>80</strong> adversarial cases</span>
              <span><strong>12</strong> threat families</span>
              <span><strong>$0</strong> unauthorized release tolerance</span>
            </div>
          </div>
          <div className="landing-next-hint" aria-hidden="true">The release gate, end to end</div>
        </section>

        <section className="landing-judge-strip">
          <div><Check size={15} /> Gemini 3.5 Flash + Google ADK</div>
          <div><Check size={15} /> Asynchronous durable workflow</div>
          <div><Check size={15} /> Deterministic Rust policy</div>
          <div><Check size={15} /> Cloud KMS signed evidence</div>
        </section>

        <section id="how-it-works" className="landing-section">
          <div className="landing-section-heading">
            <span>Why Sentinel exists</span>
            <h2>Model confidence is not production evidence.</h2>
            <p>
              Enterprise agents can trigger real payments and touch protected data. Sentinel turns release approval
              from a subjective review into a reproducible security decision tied to the exact deployable revision.
            </p>
          </div>
          <div className="landing-control-grid">
            {controls.map(({ icon: Icon, title, body }, index) => (
              <article key={title}>
                <div className="landing-control-number">0{index + 1}</div>
                <Icon size={21} aria-hidden="true" />
                <h3>{title}</h3>
                <p>{body}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="landing-replay-band">
          <div className="landing-replay-copy">
            <span>End-to-end release gate</span>
            <h2>See the autonomous decision, not a decorative dashboard.</h2>
            <p>
              Mission Control connects to the live control plane. Every workflow, audit event, and attestation
              is produced by the running system — nothing is pre-populated or fabricated in the browser.
            </p>
            <Link href="/signin">
              Enter mission control <ArrowRight size={16} aria-hidden="true" />
            </Link>
          </div>
          <div className="landing-replay-console" aria-label="Certification workflow stages">
            <div className="landing-console-head">
              <span>Certification run · AP agent v1.4.2</span>
              <strong>Live Cloud</strong>
            </div>
            <ol>
              {stages.map((stage, index) => (
                <li key={stage}>
                  <span>{index < 3 ? <Check size={13} /> : index + 1}</span>
                  <div>
                    <strong>{stage}</strong>
                    <small>{index < 3 ? 'Evidence recorded' : index === 3 ? 'Reviewer decision required' : 'Pending constraint'}</small>
                  </div>
                </li>
              ))}
            </ol>
            <div className="landing-console-result">
              <ShieldCheck size={18} />
              <div><strong>Constrained approval required</strong><span>Revoke release_payment before production</span></div>
            </div>
          </div>
        </section>

        <section id="architecture" className="landing-section landing-architecture">
          <div className="landing-section-heading">
            <span>Production-minded architecture</span>
            <h2>AI observes. Deterministic systems decide.</h2>
          </div>
          <div className="landing-architecture-flow" aria-label="System architecture">
            <div><strong>Next.js Console</strong><span>Reviewer and fleet control</span></div>
            <ArrowRight aria-hidden="true" />
            <div><strong>Rust Control Plane</strong><span>State, policy, tenancy</span></div>
            <ArrowRight aria-hidden="true" />
            <div><strong>Google ADK Certifier</strong><span>Gemini + Model Armor</span></div>
            <ArrowRight aria-hidden="true" />
            <div><strong>Evidence + KMS</strong><span>Verifiable release passport</span></div>
          </div>
        </section>

        <section className="landing-final-cta">
          <Image src="/chimera-sentinel.png" alt="" width={72} height={72} />
          <div>
            <span>One revision. One evidence trail. One production decision.</span>
            <h2>Put the candidate through the gate.</h2>
          </div>
          <Link href="/signin">Open mission control <ArrowRight size={17} /></Link>
        </section>
      </main>

      <footer className="landing-footer">
        <span>Chimera Sentinel · AI Agent Admission Control</span>
        <span>Google Cloud · Gemini · ADK · Rust</span>
      </footer>
    </div>
  );
}