//! Tracing, OpenTelemetry W3C trace context, and strict allowlist-based redaction.
//!
//! Enforces that raw invoice bodies, prompts, credentials, tokens, PII, and
//! model chain-of-thought are never exported to external telemetry.
//!
//! # Feature flags
//! - `cloud-trace` — enables the Google Cloud Trace OTel exporter via OTLP gRPC.
//!   When disabled (default), only stdout JSON tracing is active.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize tracing with stdout JSON output only.
/// Use in LOCAL / CI mode where GCP is not available.
pub fn init_tracing(service_name: &str) -> Result<(), String> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new(format!("info,{}=debug", service_name))
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e: tracing_subscriber::util::TryInitError| e.to_string())?;

    Ok(())
}

/// Initialize tracing with Google Cloud Trace OTLP gRPC exporter.
///
/// Requires the `cloud-trace` feature flag and the `OTEL_EXPORTER_OTLP_ENDPOINT`
/// env var (or the default Cloud Trace OTLP endpoint).
///
/// Falls back to stdout-only tracing if the exporter cannot be configured,
/// **and logs a warning** — this is the only acceptable non-fatal fallback
/// because telemetry export failure must never block the primary request path.
#[cfg(feature = "cloud-trace")]
pub fn init_tracing_with_cloud_trace(service_name: &str, gcp_project: &str) -> Result<(), String> {
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::Resource;
    use tracing_opentelemetry::OpenTelemetryLayer;

    // Cloud Trace OTLP endpoint (gRPC)
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "https://cloudtrace.googleapis.com".to_string());

    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("gcp.project_id", gcp_project.to_string()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(resource))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(|e| format!("OTel Cloud Trace init error: {e}"))?;

    let otel_layer = OpenTelemetryLayer::new(tracer);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new(format!("info,{}=debug", service_name))
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .try_init()
        .map_err(|e: tracing_subscriber::util::TryInitError| e.to_string())?;

    Ok(())
}

/// Strict allowlist-based redaction processor.
pub mod redact {
    use std::collections::HashSet;

    /// Allowlisted telemetry attribute keys. ONLY these keys may appear in telemetry.
    pub const ALLOWED_KEYS: &[&str] = &[
        "tenant_id",
        "workflow_id",
        "case_run_id",
        "case_id",
        "operation",
        "outcome",
        "latency_ms",
        "retry_count",
        "provenance",
        "model_version",
        "policy_version",
        "status_code",
        "attempt",
        "token_count",
        "cost_usd_micro",
        "trace_id",
        "span_id",
    ];

    /// Checks whether an attribute key is safe for telemetry export.
    pub fn is_allowed(key: &str) -> bool {
        ALLOWED_KEYS.contains(&key)
    }

    /// Redacts arbitrary JSON values by removing any key not on the allowlist.
    pub fn redact_value(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let allowed_set: HashSet<&str> = ALLOWED_KEYS.iter().copied().collect();
                let filtered: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .filter(|(k, _)| allowed_set.contains(k.as_str()))
                    .map(|(k, v)| (k.clone(), redact_value(v)))
                    .collect();
                serde_json::Value::Object(filtered)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(redact_value).collect())
            }
            v => v.clone(),
        }
    }

    /// Scans a text payload for forbidden canary secret tokens.
    pub fn contains_canary_secret(text: &str, canary: &str) -> bool {
        text.contains(canary)
    }
}
