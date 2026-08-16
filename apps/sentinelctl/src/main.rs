//! Chimera Sentinel CLI Tool (`sentinelctl`).
//!
//! Provides command-line interfaces for offline attestation verification,
//! candidate registration, workflow inspection, and seeding demo fixtures.

use clap::{Parser, Subcommand};
use sentinel_config::{load_config, SentinelCtlConfig};
use time::OffsetDateTime;

use sentinel_attestation::verify_attestation;
use sentinel_domain::{attestation::AttestationMetadata, ids::TenantId};

#[derive(Parser)]
#[command(
    name = "sentinelctl",
    version = "0.1.0",
    about = "Chimera Sentinel Release Admission CLI"
)]
struct Cli {
    #[arg(long, default_value = "/app/config/sentinelctl.toml")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Offline verification of a signed release attestation
    AttestationVerify {
        /// Path to attestation JSON file
        #[arg(short, long)]
        file: String,
        /// Expected tenant ID
        #[arg(long)]
        tenant: Option<String>,
        /// Expected target environment
        #[arg(long)]
        environment: Option<String>,
        /// Optional expected candidate revision ID
        #[arg(long)]
        revision: Option<String>,
    },
    /// Seed synthetic test data into control plane API
    Seed {
        /// Control plane API base URL
        #[arg(long)]
        api_url: Option<String>,
        /// Target tenant ID
        #[arg(long)]
        tenant: Option<String>,
    },
    /// Inspect health and status of control plane
    Health {
        #[arg(long)]
        api_url: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config: SentinelCtlConfig =
        load_config(&cli.config).map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

    // Initialize structured logging
    sentinel_config::init_logging(&config.observability)?;

    match cli.command {
        Commands::AttestationVerify {
            file,
            tenant,
            environment,
            revision,
        } => {
            println!("==> Chimera Sentinel Offline Attestation Verifier");
            println!("Reading attestation from: {}", file);

            let content = std::fs::read_to_string(&file)?;
            let attestation: AttestationMetadata = serde_json::from_str(&content)?;
            let tenant_id = TenantId::parse(
                tenant
                    .as_deref()
                    .unwrap_or(&config.attestation_verify.default_tenant),
            )?;
            let env = environment
                .as_deref()
                .unwrap_or(&config.attestation_verify.default_environment);
            let now = OffsetDateTime::now_utc();

            let result = verify_attestation(
                &attestation,
                &tenant_id,
                env,
                revision.as_deref(),
                None,
                now,
            );

            println!("\nVerification Checks:");
            println!(
                "  [1] Cryptographic Signature: {}",
                if result.checks.signature_valid {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
            println!(
                "  [2] KMS Key Reference Match: {}",
                if result.checks.key_matches {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
            println!(
                "  [3] Validity Window (Not Expired): {}",
                if result.checks.not_expired {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
            println!(
                "  [4] Revocation Status Check: {}",
                if result.checks.not_revoked {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
            println!(
                "  [5] Tenant Audience Match: {}",
                if result.checks.audience_matches {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
            println!(
                "  [6] Target Environment Match: {}",
                if result.checks.environment_matches {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
            println!(
                "  [7] Candidate Revision Match: {}",
                if result.checks.revision_matches {
                    "PASS"
                } else {
                    "FAIL"
                }
            );
            println!(
                "  [8] Evidence Manifest Digest: {}",
                if result.checks.digest_matches {
                    "PASS"
                } else {
                    "FAIL"
                }
            );

            if result.valid {
                println!("\nSUCCESS: Attestation is VALID and cryptographic signature verified.");
                if let Some(payload) = &result.payload {
                    println!("  Candidate Revision: {}", payload.candidate_revision_id);
                    println!("  Agent Identity: {}", payload.agent_identity);
                    println!("  Model Reference: {}", payload.model_ref);
                    println!("  Decision: {:?}", payload.decision);
                    println!("  Expires At: {}", payload.expires_at);
                }
            } else {
                eprintln!("\nFAILURE: Attestation verification FAILED!");
                if let Some(err) = &result.error {
                    eprintln!("  Errors: {}", err);
                }
                std::process::exit(1);
            }
        }
        Commands::Seed { api_url, tenant } => {
            let api = api_url.unwrap_or_else(|| "http://localhost:8080".to_string());
            let tenant_id = TenantId::parse(
                tenant
                    .as_deref()
                    .unwrap_or(&config.attestation_verify.default_tenant),
            )?;

            println!("==> Seeding synthetic test fixtures to: {}", api);
            let candidate = sentinel_test_support::test_candidate_revision(
                tenant_id,
                vec![
                    "draft_invoice_payment".to_string(),
                    "release_payment".to_string(),
                ],
            );

            let client = reqwest::Client::new();
            let res = client
                .post(format!("{}/v1/candidates", api))
                .header("X-Tenant-ID", tenant_id.to_string())
                .json(&serde_json::json!({
                    "tenant_id": tenant_id,
                    "agent_id": candidate.agent_id,
                    "abom": candidate.abom,
                    "policy_pack_id": candidate.policy_pack_id,
                    "corpus_version": candidate.corpus_version,
                }))
                .send()
                .await;

            match res {
                Ok(resp) if resp.status().is_success() => {
                    println!(
                        "Candidate successfully registered with revision ID: {}",
                        candidate.revision_id
                    );
                }
                Ok(resp) => {
                    eprintln!("Failed to register candidate: status {}", resp.status());
                }
                Err(e) => {
                    eprintln!("Failed to connect to API server: {}", e);
                }
            }
        }
        Commands::Health { api_url } => {
            let api = api_url.unwrap_or_else(|| "http://localhost:8080".to_string());
            println!("Checking health of API server at: {}", api);
            let client = reqwest::Client::new();
            match client.get(format!("{}/healthz", api)).send().await {
                Ok(resp) if resp.status().is_success() => println!("Health: OK (200)"),
                Ok(resp) => eprintln!("Health check returned status: {}", resp.status()),
                Err(e) => eprintln!("Failed to reach health endpoint: {}", e),
            }
        }
    }

    Ok(())
}

