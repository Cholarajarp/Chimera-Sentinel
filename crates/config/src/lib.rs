#![allow(missing_docs)]
//! Shared configuration for all Sentinel services.
//!
//! Provides TOML file + environment variable configuration with validation.

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration parsing failed: {0}")]
    Parse(#[from] figment::Error),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Load configuration from file and environment variables.
/// Environment variables are prefixed with `SENTINEL_` and use `__` for nesting.
/// Example: SENTINEL_SERVER__ADDR="0.0.0.0:8080"
pub fn load_config<T: for<'de> Deserialize<'de>>(config_path: &str) -> Result<T, ConfigError> {
    let figment = Figment::new()
        .merge(Toml::file(config_path))
        .merge(Env::prefixed("SENTINEL_").split("__"));

    Ok(figment.extract()?)
}

/// Common server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub addr: String,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:8080".to_string(),
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

/// Database configuration
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DatabaseConfig {
    #[default]
    Memory,
    Firestore {
        project_id: String,
        database_id: Option<String>,
    },
    Postgres {
        url: String,
        max_connections: Option<u32>,
    },
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub log_format: LogFormat,
    pub otel_endpoint: String,
    pub otel_service_name: String,
    pub trace_sample_rate: f64,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_format: LogFormat::Json,
            otel_endpoint: "http://localhost:4317".to_string(),
            otel_service_name: "sentinel".to_string(),
            trace_sample_rate: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub rate_limit_rps: u32,
    pub rate_limit_burst: u32,
    pub require_auth: bool,
    pub jwks_url: Option<String>,
    pub allowed_tenants: Option<Vec<String>>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit_rps: 100,
            rate_limit_burst: 200,
            require_auth: false,
            jwks_url: None,
            allowed_tenants: None,
        }
    }
}

/// Deserializes a string field that may arrive as a bare number.
///
/// Environment values like `KMS_KEY_VERSION=1` are parsed as integers before
/// reaching serde, which would otherwise fail the whole configuration.
fn de_flexible_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, Visitor};
    use std::fmt;

    struct FlexibleString;

    impl Visitor<'_> for FlexibleString {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or an integer")
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            Ok(value.to_owned())
        }

        fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value.to_string())
        }

        fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(FlexibleString)
}

/// Google Cloud configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GoogleCloudConfig {
    pub use_live_services: bool,
    pub model_armor_template: String,
    pub gateway_policy: String,
    pub kms_key_resource: String,
    #[serde(deserialize_with = "de_flexible_string")]
    pub kms_key_version: String,
    pub firestore_project: String,
    pub storage_bucket: String,
}

impl Default for GoogleCloudConfig {
    fn default() -> Self {
        Self {
            use_live_services: false,
            model_armor_template: "projects/chimera-sentinel/locations/us-east1/templates/armor-template-v1".to_string(),
            gateway_policy: "projects/chimera-sentinel/locations/us-east1/gatewayPolicies/ap-least-privilege-v1".to_string(),
            kms_key_resource: "projects/chimera-sentinel/locations/us-east1/keyRings/sentinel-ring/cryptoKeys/attestation-signer".to_string(),
            kms_key_version: "1".to_string(),
            firestore_project: "chimera-sentinel".to_string(),
            storage_bucket: "chimera-sentinel-evidence-bucket".to_string(),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HealthCheckConfig {
    pub check_database: bool,
    pub check_google_services: bool,
    pub check_erp_adapter: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_database: true,
            check_google_services: false,
            check_erp_adapter: true,
        }
    }
}

/// Control Plane specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ControlPlaneConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub observability: ObservabilityConfig,
    pub security: SecurityConfig,
    pub google_cloud: GoogleCloudConfig,
    pub worker: WorkerConfig,
    pub health_checks: HealthCheckConfig,
}

impl Default for ControlPlaneConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            observability: ObservabilityConfig {
                otel_service_name: "sentinel-control-plane".to_string(),
                ..Default::default()
            },
            security: SecurityConfig::default(),
            google_cloud: GoogleCloudConfig::default(),
            worker: WorkerConfig::default(),
            health_checks: HealthCheckConfig::default(),
        }
    }
}

/// Workflow Worker specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// A present-but-partial [worker] table must not fail the whole config.
#[serde(default)]
pub struct WorkerConfig {
    pub id: String,
    pub tenant_id: String,
    pub poll_interval_secs: u64,
    pub max_concurrent_workflows: usize,
    pub shutdown_timeout_secs: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            id: "worker-1".to_string(),
            tenant_id: "00000000-0000-0000-0000-000000000001".to_string(),
            poll_interval_secs: 5,
            max_concurrent_workflows: 10,
            shutdown_timeout_secs: 30,
        }
    }
}

/// Workflow Worker specific configuration (extended)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkflowWorkerConfig {
    pub worker: WorkerConfig,
    pub database: DatabaseConfig,
    pub observability: ObservabilityConfig,
    pub google_cloud: GoogleCloudConfig,
    pub certification: CertificationConfig,
    pub health_checks: HealthCheckConfig,
}

impl Default for WorkflowWorkerConfig {
    fn default() -> Self {
        Self {
            worker: WorkerConfig::default(),
            database: DatabaseConfig::default(),
            observability: ObservabilityConfig {
                otel_service_name: "sentinel-workflow-worker".to_string(),
                ..Default::default()
            },
            google_cloud: GoogleCloudConfig::default(),
            certification: CertificationConfig::default(),
            health_checks: HealthCheckConfig::default(),
        }
    }
}

/// Certification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationConfig {
    pub corpus_version: String,
    pub required_cases: Vec<String>,
    pub retest_draft_case: String,
    pub retest_release_case: String,
}

impl Default for CertificationConfig {
    fn default() -> Self {
        Self {
            corpus_version: "corpus-v1".to_string(),
            required_cases: vec![
                "case-safe-draft-001".to_string(),
                "case-prompt-inj-direct-001".to_string(),
                "case-agency-release-attempt-001".to_string(),
            ],
            retest_draft_case: "retest-draft-001".to_string(),
            retest_release_case: "retest-release-denied-001".to_string(),
        }
    }
}

/// Enterprise ERP Adapter MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EnterpriseErpConfig {
    pub server: ServerConfig,
    pub ledger: LedgerConfig,
    pub observability: ObservabilityConfig,
    pub security: SecurityConfig,
}

impl Default for EnterpriseErpConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                addr: "0.0.0.0:9090".to_string(),
                ..Default::default()
            },
            ledger: LedgerConfig::default(),
            observability: ObservabilityConfig {
                otel_service_name: "sentinel-enterprise-erp-adapter".to_string(),
                ..Default::default()
            },
            security: SecurityConfig {
                rate_limit_rps: 50,
                rate_limit_burst: 100,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerConfig {
    pub initial_payments: Vec<String>,
    pub enforce_zero_unauthorized: bool,
    pub enforce_no_duplicates: bool,
    pub enforce_authorization: bool,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            initial_payments: vec![],
            enforce_zero_unauthorized: true,
            enforce_no_duplicates: true,
            enforce_authorization: true,
        }
    }
}

/// Sentinel CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SentinelCtlConfig {
    pub cli: CliConfig,
    pub attestation_verify: AttestationVerifyConfig,
    pub google_cloud: GoogleCloudConfig,
    pub observability: ObservabilityConfig,
}

impl Default for SentinelCtlConfig {
    fn default() -> Self {
        Self {
            cli: CliConfig::default(),
            attestation_verify: AttestationVerifyConfig::default(),
            google_cloud: GoogleCloudConfig::default(),
            observability: ObservabilityConfig {
                log_level: "warn".to_string(),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub output_format: OutputFormat,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Json,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Yaml,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationVerifyConfig {
    pub default_tenant: String,
    pub default_environment: String,
}

impl Default for AttestationVerifyConfig {
    fn default() -> Self {
        Self {
            default_tenant: "00000000-0000-0000-0000-000000000001".to_string(),
            default_environment: "production".to_string(),
        }
    }
}

/// Initialize structured logging based on config
pub fn init_logging(config: &ObservabilityConfig) -> Result<(), ConfigError> {
    use tracing_subscriber::{
        fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let fmt_layer = match config.log_format {
        LogFormat::Json => fmt::layer()
            .json()
            .with_span_list(true)
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .boxed(),
        LogFormat::Pretty => fmt::layer()
            .pretty()
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .boxed(),
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_control_plane_config() {
        let toml = r#"
            [server]
            addr = "0.0.0.0:8080"

            [database]
            type = "memory"

            [observability]
            log_level = "debug"
            log_format = "json"
            otel_endpoint = "http://localhost:4317"
            otel_service_name = "test"
            trace_sample_rate = 0.5

            [security]
            rate_limit_rps = 50
            rate_limit_burst = 100
            require_auth = true
        "#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(toml.as_bytes()).unwrap();
        file.flush().unwrap();

        let config: ControlPlaneConfig = load_config(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.server.addr, "0.0.0.0:8080");
        assert_eq!(config.observability.log_level, "debug");
        assert_eq!(config.security.rate_limit_rps, 50);
    }
}
