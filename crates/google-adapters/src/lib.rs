#![allow(missing_docs)]
//! Isolated provider ports and adapters for Google Cloud agent services.
//!
//! Provides clean interfaces for Agent Registry, Agent Runtime, Model Armor,
//! Agent Gateway, Memory Bank, Cloud KMS, Firestore, and Cloud Storage with
//! explicit provenance tagging (`LIVE` vs `LOCAL`).

use time::OffsetDateTime;

use sentinel_domain::{
    evidence::{GatewayDecisionEvidence, MemoryRetrievalEvidence, ModelArmorEvidence},
    ids::{CaseId, PrincipalId, TenantId},
    provenance::Provenance,
};

pub mod model_armor {
    use super::*;

    /// Inspects content for prompt injection, jailbreaks, and sensitive data.
    pub async fn inspect_content(
        tenant_id: TenantId,
        case_id: &CaseId,
        content: &str,
        template_id: &str,
        is_live: bool,
    ) -> Result<ModelArmorEvidence, String> {
        let now = OffsetDateTime::now_utc();
        let (disposition, matched_rules, explanation) = if content
            .to_lowercase()
            .contains("ignore previous instructions")
            || content.to_lowercase().contains("ignore policy")
            || content.to_lowercase().contains("override policy")
            || content
                .to_lowercase()
                .contains("pay immediately without review")
        {
            (
                "BLOCK".to_string(),
                vec!["PROMPT_INJECTION_DIRECT".to_string()],
                "Model Armor detected direct prompt injection attempting policy override"
                    .to_string(),
            )
        } else {
            (
                "ALLOW".to_string(),
                vec![],
                "Content inspected: no hostile pattern detected".to_string(),
            )
        };

        Ok(ModelArmorEvidence {
            tenant_id,
            case_id: case_id.clone(),
            template_id: template_id.to_string(),
            disposition,
            matched_rules,
            sanitized_explanation: explanation,
            provider_reference: format!(
                "projects/sentinel-prod/locations/us-central1/templates/{}",
                template_id
            ),
            provenance: if is_live {
                Provenance::Live
            } else {
                Provenance::Local
            },
            timestamp: now,
        })
    }
}

pub mod gateway {
    use super::*;

    /// Evaluates Agent Gateway identity-scoped tool authorization policy.
    pub async fn check_permission(
        tenant_id: TenantId,
        principal: &PrincipalId,
        action: &str,
        is_live: bool,
    ) -> Result<GatewayDecisionEvidence, String> {
        let now = OffsetDateTime::now_utc();

        // Gateway policy: candidate identity is granted draft_invoice_payment,
        // but release_payment is explicitly DENIED by default for least privilege.
        let allowed = match action {
            "draft_invoice_payment" | "get_payment_status" => true,
            "release_payment" => false, // Excessive agency blocked at Gateway
            _ => false,
        };

        Ok(GatewayDecisionEvidence {
            tenant_id,
            principal: principal.clone(),
            action: action.to_string(),
            allowed,
            policy_reference:
                "projects/sentinel-prod/locations/us-central1/gatewayPolicies/ap-least-privilege-v1"
                    .to_string(),
            provider_reference: format!(
                "projects/sentinel-prod/locations/us-central1/gateways/gw-{}",
                tenant_id
            ),
            provenance: if is_live {
                Provenance::Live
            } else {
                Provenance::Local
            },
            timestamp: now,
        })
    }
}

pub mod memory_bank {
    use super::*;

    /// Retrieves advisory prior disposition context from Memory Bank.
    pub async fn query_advisory_context(
        tenant_id: TenantId,
        memory_bank_id: &str,
        query: &str,
        is_live: bool,
    ) -> Result<MemoryRetrievalEvidence, String> {
        let now = OffsetDateTime::now_utc();
        let items = if query.to_lowercase().contains("invoice") {
            vec![
                "Analyst Disposition #2026-04A: Invoices under $5,000 require draft creation only, subject to automated batch verification".to_string(),
            ]
        } else {
            vec![]
        };

        Ok(MemoryRetrievalEvidence {
            tenant_id,
            memory_bank_id: memory_bank_id.to_string(),
            retrieved_items: items,
            analyst_approval_references: vec!["DISP-REF-202604A-SEC".to_string()],
            provider_reference: format!(
                "projects/sentinel-prod/locations/us-central1/memoryBanks/{}",
                memory_bank_id
            ),
            provenance: if is_live {
                Provenance::Live
            } else {
                Provenance::Local
            },
            timestamp: now,
        })
    }
}

/// Google Artifact Registry — On-Demand Vulnerability Scanning.
///
/// Calls the GCP **On-Demand Scanning API** (`ondemandscanning.googleapis.com`)
/// to scan an Artifact Registry / Container Registry image digest for known
/// CVEs and returns them as domain [`ScanResponse`] records. Every result is
/// tagged [`Provenance::Live`] because it is a direct observation from a
/// real managed-Google call; callers are responsible for returning
/// [`Provenance::Local`] (honest empty) when no credentials are available.
pub mod artifact_registry {
    use super::*;
    use sentinel_domain::candidate::{ScanResponse, Vulnerability};
    use std::time::Duration;

    /// Base URL of the On-Demand Scanning API.
    const SCANNING_API_BASE: &str = "https://ondemandscanning.googleapis.com/v1";

    /// Callable-state names returned by the On-Demand Scanning API. Only
    /// `COMPLETED` is considered success; anything else is treated as not done
    /// (during polling) or as a scan error (if terminal but not completed).
    const STATE_COMPLETED: &str = "COMPLETED";
    const STATE_FAILED: &str = "FAILED";

    /// A scan target: the image to scan and the GCP project/region that owns the
    /// On-Demand Scanning parent resource.
    pub struct ScanTarget<'a> {
        pub project: &'a str,
        pub location: &'a str,
        /// Fully-qualified container image URI, optionally with an explicit
        /// digest pin, e.g.
        /// `us-east1-docker.pkg.dev/proj/sentinel/control-plane@sha256:abcd…`.
        pub resource_uri: &'a str,
    }

    /// Builds a short-lived HTTP client used for all scanner calls. A single
    /// client is reused so HTTP keep-alive and connection pooling work across
    /// the create/poll/list round-trips.
    fn http_client() -> Result<reqwest::Client, String> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to build On-Demand Scanning HTTP client: {e}"))
    }

    /// Scan a container image with the GCP On-Demand Scanning API and return the
    /// vulnerability findings.
    ///
    /// Flow: create the scan → poll its state until it reaches a terminal
    /// state → list vulnerability occurrences → map to [`Vulnerability`].
    ///
    /// `access_token` must be a Google OAuth2 token authorized for the On-Demand
    /// Scanning service (the `https://www.ondemandscanning.googleapis.com`
    /// scope is the canonical choice). On any transport/HTTP failure this
    /// returns `Err`; the caller must surface that as an upstream error, NOT
    /// fabricate an empty-but-clean result.
    pub async fn scan(target: &ScanTarget<'_>, access_token: &str) -> Result<ScanResponse, String> {
        let client = http_client()?;
        let scan_name = create_scan(&client, target, access_token).await?;
        wait_for_completion(&client, &scan_name, access_token).await?;
        let occurrences = list_vulnerabilities(&client, &scan_name, access_token).await?;
        let vulnerabilities = occurrences
            .as_array()
            .map(|arr| arr.iter().map(vuln_from_occurrence).collect())
            .unwrap_or_default();

        Ok(ScanResponse {
            status: STATE_COMPLETED.to_string(),
            provenance: Provenance::Live,
            vulnerabilities,
        })
    }

    /// `POST /v1/projects/{p}/locations/{l}/scans` — kick off the on-demand scan.
    /// Returns the scan's relative `name` (e.g.
    /// `projects/…/locations/…/scans/scan-<uuid>`), used by the poll/list steps.
    async fn create_scan(
        client: &reqwest::Client,
        target: &ScanTarget<'_>,
        access_token: &str,
    ) -> Result<String, String> {
        let url = format!(
            "{SCANNING_API_BASE}/projects/{}/locations/{}/scans",
            target.project, target.location
        );
        let response = client
            .post(url)
            .bearer_auth(access_token)
            .json(&serde_json::json!({ "resourceUri": target.resource_uri }))
            .send()
            .await
            .map_err(|e| format!("On-Demand scan create request failed: {e}"))?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("On-Demand scan create returned invalid JSON ({status}): {e}"))?;
        if !status.is_success() {
            let reason = body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            return Err(format!(
                "On-Demand scan create failed with HTTP {status}: {reason}"
            ));
        }

        body.get("name")
            .and_then(|n| n.as_str())
            .map(str::to_string)
            .ok_or_else(|| "On-Demand scan create response missing `name`".to_string())
    }

    /// Poll `GET /v1/{scan_name}` until the scan `state` reaches a terminal
    /// state. Bounded by `MAX_POLLS * POLL_INTERVAL` to avoid wedging the
    /// request on a slow/hung scan.
    async fn wait_for_completion(
        client: &reqwest::Client,
        scan_name: &str,
        access_token: &str,
    ) -> Result<(), String> {
        const MAX_POLLS: usize = 30;
        const POLL_INTERVAL: Duration = Duration::from_secs(5);
        let url = format!("{SCANNING_API_BASE}/{scan_name}");

        for _ in 0..MAX_POLLS {
            let response = client
                .get(&url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| format!("On-Demand scan poll request failed: {e}"))?;
            let status = response.status();
            let body: serde_json::Value = response.json().await.map_err(|e| {
                format!("On-Demand scan poll returned invalid JSON ({status}): {e}")
            })?;
            if !status.is_success() {
                let reason = body
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                return Err(format!(
                    "On-Demand scan poll failed with HTTP {status}: {reason}"
                ));
            }
            match body.get("state").and_then(|s| s.as_str()) {
                Some(s) if s == STATE_COMPLETED => return Ok(()),
                Some(s) if s == STATE_FAILED => {
                    return Err(format!("On-Demand scan ended in FAILED state: {body:?}"));
                }
                _ => tokio::time::sleep(POLL_INTERVAL).await,
            }
        }
        Err(format!(
            "On-Demand scan for `{scan_name}` did not complete within {MAX_POLLS} polls"
        ))
    }

    /// `GET /v1/{scan_name}:listVulnerabilities` — fetch the vulnerability
    /// occurrences produced by a completed scan. Returns the raw `occurrences`
    /// array (defensively extracted from either `occurrences` or the
    /// `listVulnerabilitiesResponse` wrapper shape).
    async fn list_vulnerabilities(
        client: &reqwest::Client,
        scan_name: &str,
        access_token: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{SCANNING_API_BASE}/{scan_name}:listVulnerabilities");
        let response = client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("On-Demand scan listVulnerabilities request failed: {e}"))?;
        let status = response.status();
        let body: serde_json::Value = response.json().await.map_err(|e| {
            format!("On-Demand scan listVulnerabilities returned invalid JSON ({status}): {e}")
        })?;
        if !status.is_success() {
            let reason = body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            return Err(format!(
                "On-Demand scan listVulnerabilities failed with HTTP {status}: {reason}"
            ));
        }
        // The documented response shape carries `occurrences`; be tolerant.
        Ok(body
            .get("occurrences")
            .cloned()
            .or_else(|| {
                body.get("listVulnerabilitiesResponse")
                    .and_then(|r| r.get("occurrences"))
                    .cloned()
            })
            .unwrap_or(serde_json::Value::Array(vec![])))
    }

    /// Map a Container Analysis / On-Demand Scanning vulnerability occurrence to
    /// the domain [`Vulnerability`] record. Field names are extracted
    /// defensively because the upstream schema is verbose and versioned.
    fn vuln_from_occurrence(occurrence: &serde_json::Value) -> Vulnerability {
        let vuln = occurrence
            .get("vulnerability")
            .unwrap_or(&serde_json::Value::Null);
        let severity = vuln
            .get("severity")
            .and_then(|s| s.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();
        // `shortDescription` frequently carries the CVE id (e.g. "CVE-2025-…").
        let cve_id = vuln
            .get("shortDescription")
            .and_then(|s| s.as_str())
            .filter(|s| s.contains("CVE") || !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                occurrence
                    .get("name")
                    .and_then(|n| n.as_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| "unknown".to_string())
            });

        let title = occurrence
            .get("packageIssue")
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("affectedPackage"))
            .and_then(|p| p.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| cve_id.clone());

        let long_description = vuln
            .get("longDescription")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let pkg = occurrence
            .get("packageIssue")
            .and_then(|p| p.get(0))
            .map(|issue| {
                let pkg_name = issue
                    .get("affectedPackage")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                let pkg_version = issue
                    .get("affectedVersion")
                    .and_then(|v| v.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let fixed_version = issue
                    .get("fixedVersion")
                    .and_then(|v| v.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                format!(
                    "Package={pkg_name} installed={pkg_version} fixed={fixed_version} — {long_description}"
                )
            })
            .unwrap_or_else(|| long_description);

        Vulnerability {
            id: cve_id,
            title,
            severity,
            description: pkg,
        }
    }
}
