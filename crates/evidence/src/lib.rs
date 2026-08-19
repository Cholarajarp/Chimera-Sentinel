//! Canonicalization, SHA-256 digesting, and content-addressed evidence manifests.
//!
//! Implements JSON Canonicalization Scheme (JCS / RFC 8785) for deterministic
//! cryptographic digests over evidence objects and attestation payloads.

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use time::OffsetDateTime;

use sentinel_domain::{
    evidence::{EvidenceManifestRef, EvidenceObjectRef},
    ids::{TenantId, WorkflowId},
    provenance::Provenance,
};

/// Canonicalizes a serde serializable value to bytes per RFC 8785 (JCS).
pub fn canonicalize<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let json_val = serde_json::to_value(value)?;
    let canonical_str = canonicalize_value(&json_val);
    Ok(canonical_str.into_bytes())
}

/// Recursively formats a serde_json::Value into an RFC 8785 canonical string.
pub fn canonicalize_value(val: &Value) -> String {
    match val {
        Value::Null => "null".to_string(),
        Value::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(n) => n.to_string(),
        Value::String(s) => escape_json_string(s),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(canonicalize_value).collect();
            format!("[{}]", elements.join(","))
        }
        Value::Object(map) => {
            // RFC 8785: Keys MUST be sorted lexicographically by UTF-16 code units / UTF-8 bytes
            let sorted: BTreeMap<&str, &Value> = map.iter().map(|(k, v)| (k.as_str(), v)).collect();
            let entries: Vec<String> = sorted
                .into_iter()
                .map(|(k, v)| format!("{}:{}", escape_json_string(k), canonicalize_value(v)))
                .collect();
            format!("{{{}}}", entries.join(","))
        }
    }
}

fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\x08' => out.push_str("\\b"),
            '\x0C' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Computes the SHA-256 digest of arbitrary bytes with "sha256:" prefix.
pub fn sha256_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

/// Computes the canonical SHA-256 digest of any serializable object.
pub fn canonical_digest<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let canonical_bytes = canonicalize(value)?;
    Ok(sha256_digest(&canonical_bytes))
}

/// Builder for constructing verified content-addressed evidence manifests.
pub struct EvidenceManifestBuilder {
    tenant_id: TenantId,
    workflow_id: WorkflowId,
    objects: Vec<EvidenceObjectRef>,
}

impl EvidenceManifestBuilder {
    pub fn new(tenant_id: TenantId, workflow_id: WorkflowId) -> Self {
        Self {
            tenant_id,
            workflow_id,
            objects: Vec::new(),
        }
    }

    /// Add an evidence object after computing its canonical hash and size.
    pub fn add_object<T: Serialize>(
        &mut self,
        path: impl Into<String>,
        media_type: impl Into<String>,
        schema_version: impl Into<String>,
        producer: impl Into<String>,
        provenance: Provenance,
        content: &T,
        timestamp: OffsetDateTime,
    ) -> Result<&mut Self, serde_json::Error> {
        let canonical_bytes = canonicalize(content)?;
        let digest = sha256_digest(&canonical_bytes);
        let size_bytes = canonical_bytes.len() as u64;

        self.objects.push(EvidenceObjectRef {
            path: path.into(),
            media_type: media_type.into(),
            schema_version: schema_version.into(),
            sha256_digest: digest,
            size_bytes,
            producer: producer.into(),
            provenance,
            timestamp,
        });

        Ok(self)
    }

    /// Build the finalized immutable EvidenceManifestRef.
    pub fn build(self, now: OffsetDateTime) -> EvidenceManifestRef {
        let manifest_digest = EvidenceManifestRef::compute_digest(&self.objects);
        EvidenceManifestRef {
            schema_version: "sentinel.manifest.v1".to_string(),
            tenant_id: self.tenant_id,
            workflow_id: self.workflow_id,
            objects: self.objects,
            manifest_digest,
            created_at: now,
        }
    }
}
