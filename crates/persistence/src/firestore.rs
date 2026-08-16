//! Firestore-backed repository implementation.
//!
//! Uses the Firestore REST API v1 with Application Default Credentials (workload
//! identity on Cloud Run, `gcloud auth application-default login` locally).
//!
//! Collection layout under `projects/{project}/databases/(default)/documents/`:
//!   `tenants/{tenant_id}/candidates/{revision_id}`
//!   `tenants/{tenant_id}/workflows/{workflow_id}`
//!   `tenants/{tenant_id}/evidence_manifests/{workflow_id}`
//!   `tenants/{tenant_id}/audit_events/{event_id}`
//!   `tenants/{tenant_id}/findings/{finding_id}`
//!   `tenants/{tenant_id}/approvals/{approval_id}`
//!   `tenants/{tenant_id}/attestations/{workflow_id}`
//!
//! Optimistic concurrency uses Firestore precondition transactions (update_time).
//! Tenant isolation is enforced at the collection path level — cross-tenant paths
//! are structurally impossible given the routing logic.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use sentinel_domain::{
    attestation::AttestationMetadata,
    candidate::CandidateRevision,
    evidence::{Approval, EvidenceManifestRef},
    finding::Finding,
    ids::{ApprovalId, RevisionId, TenantId, WorkflowId},
    workflow::{Workflow, WorkflowState, WorkflowSummary},
    AuditEvent,
};

use super::{
    ApprovalRepository, AttestationRepository, AuditRepository, CandidateRepository,
    EvidenceRepository, FindingRepository, WorkflowRepository,
};

// ─── GCP Auth token ───────────────────────────────────────────────────────────

/// Fetch a Bearer token from the GCE metadata server or gcloud CLI.
/// On Cloud Run this uses workload identity automatically.
async fn fetch_access_token(client: &Client) -> Result<String, String> {
    // Try GCE/Cloud Run metadata server first
    let resp = client
        .get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
        .header("Metadata-Flavor", "Google")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await;

    if let Ok(r) = resp {
        if r.status().is_success() {
            let body: Value = r.json().await.map_err(|e| e.to_string())?;
            if let Some(token) = body["access_token"].as_str() {
                debug!("Obtained GCP token from metadata server");
                return Ok(token.to_string());
            }
        }
    }

    // Fallback: gcloud CLI (local development) — blocking is fine here, only used locally
    let output = std::process::Command::new("gcloud")
        .args(["auth", "application-default", "print-access-token"])
        .output()
        .map_err(|e| format!("gcloud not found: {e}. On Cloud Run ensure workload identity is configured."))?;

    if output.status.success() {
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!("Obtained GCP token from gcloud CLI");
        return Ok(token);
    }

    Err(
        "Cannot obtain GCP access token. On Cloud Run, assign a service account with \
         roles/datastore.user. Locally, run: gcloud auth application-default login"
            .to_string(),
    )
}

// ─── Firestore REST value encoding ────────────────────────────────────────────

/// Encode a serde_json value into a Firestore field value map.
fn encode_value(v: &Value) -> Value {
    match v {
        Value::Null => json!({"nullValue": null}),
        Value::Bool(b) => json!({"booleanValue": b}),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                json!({"integerValue": i.to_string()})
            } else {
                json!({"doubleValue": n.as_f64().unwrap_or(0.0)})
            }
        }
        Value::String(s) => json!({"stringValue": s}),
        Value::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(encode_value).collect();
            json!({"arrayValue": {"values": values}})
        }
        Value::Object(map) => {
            let fields: serde_json::Map<String, Value> =
                map.iter().map(|(k, v)| (k.clone(), encode_value(v))).collect();
            json!({"mapValue": {"fields": fields}})
        }
    }
}

/// Decode a Firestore field value map into a serde_json Value.
fn decode_value(fv: &Value) -> Value {
    if let Some(s) = fv.get("stringValue") {
        return s.clone();
    }
    if let Some(b) = fv.get("booleanValue") {
        return b.clone();
    }
    if let Some(i) = fv.get("integerValue") {
        // Firestore returns integers as strings
        if let Some(s) = i.as_str() {
            if let Ok(n) = s.parse::<i64>() {
                return Value::Number(n.into());
            }
        }
        return i.clone();
    }
    if let Some(d) = fv.get("doubleValue") {
        return d.clone();
    }
    if fv.get("nullValue").is_some() {
        return Value::Null;
    }
    if let Some(av) = fv.get("arrayValue") {
        let values = av["values"]
            .as_array()
            .map(|arr| arr.iter().map(decode_value).collect())
            .unwrap_or_default();
        return Value::Array(values);
    }
    if let Some(mv) = fv.get("mapValue") {
        if let Some(fields) = mv["fields"].as_object() {
            let decoded: serde_json::Map<String, Value> =
                fields.iter().map(|(k, v)| (k.clone(), decode_value(v))).collect();
            return Value::Object(decoded);
        }
    }
    Value::Null
}

/// Encode a serializable Rust value as a Firestore document fields map.
fn to_firestore_fields<T: Serialize>(value: &T) -> Result<Value, String> {
    let json_val = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let obj = json_val.as_object().ok_or("Value must serialize to a JSON object")?;
    let fields: serde_json::Map<String, Value> =
        obj.iter().map(|(k, v)| (k.clone(), encode_value(v))).collect();
    Ok(Value::Object(fields))
}

/// Decode Firestore document fields map into a Rust value.
fn from_firestore_doc<T: for<'de> Deserialize<'de>>(doc: &Value) -> Result<T, String> {
    let fields = doc.get("fields").ok_or("Document has no 'fields'")?;
    let fields_obj = fields.as_object().ok_or("Fields is not an object")?;
    let decoded: serde_json::Map<String, Value> =
        fields_obj.iter().map(|(k, v)| (k.clone(), decode_value(v))).collect();
    serde_json::from_value(Value::Object(decoded)).map_err(|e| {
        format!("Failed to deserialize Firestore document: {e}")
    })
}

// ─── FirestoreStore ───────────────────────────────────────────────────────────

/// Firestore-backed implementation of all repository traits.
///
/// Thread-safe (`Clone + Send + Sync`). Each call fetches a fresh token; in
/// production Cloud Run refreshes happen via the metadata server with sub-second
/// latency. For high-throughput use, replace with a cached token refresher.
#[derive(Clone)]
pub struct FirestoreStore {
    client: Client,
    project_id: Arc<String>,
    database_id: Arc<String>,
}

impl FirestoreStore {
    /// Create a new store targeting the given GCP project.
    /// `database_id` is `(default)` unless you created a named database.
    pub fn new(project_id: impl Into<String>, database_id: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("failed to build HTTP client"),
            project_id: Arc::new(project_id.into()),
            database_id: Arc::new(database_id.into()),
        }
    }

    /// Root URL for the Firestore REST API.
    fn base_url(&self) -> String {
        format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/{}/documents",
            self.project_id, self.database_id
        )
    }

    /// Collection path scoped to a tenant + collection name.
    fn collection_path(&self, tenant_id: TenantId, collection: &str) -> String {
        format!(
            "{}/tenants/{}/{}",
            self.base_url(),
            tenant_id,
            collection
        )
    }

    /// Full document path scoped to tenant + collection + document ID.
    fn doc_path(&self, tenant_id: TenantId, collection: &str, doc_id: &str) -> String {
        format!(
            "{}/tenants/{}/{}/{}",
            self.base_url(),
            tenant_id,
            collection,
            doc_id
        )
    }

    /// GET a single document. Returns `None` if 404, `Err` on other failures.
    async fn get_doc(&self, path: &str, token: &str) -> Result<Option<Value>, String> {
        let resp = self
            .client
            .get(path)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Firestore GET failed: {e}"))?;

        match resp.status().as_u16() {
            200 => {
                let doc: Value = resp.json().await.map_err(|e| e.to_string())?;
                Ok(Some(doc))
            }
            404 => Ok(None),
            status => {
                let body = resp.text().await.unwrap_or_default();
                Err(format!("Firestore GET {path} returned {status}: {body}"))
            }
        }
    }

    /// CREATE or PATCH (upsert) a document. Uses PATCH which is idempotent.
    async fn upsert_doc(&self, path: &str, fields: Value, token: &str) -> Result<(), String> {
        let body = json!({"fields": fields});
        let resp = self
            .client
            .patch(path)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Firestore PATCH failed: {e}"))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("Firestore PATCH {path} returned {status}: {body}"))
        }
    }

    /// UPDATE with optimistic concurrency using `currentDocument.updateTime`.
    /// If `update_time` is `None`, falls back to unconditional upsert.
    async fn update_doc_versioned(
        &self,
        path: &str,
        fields: Value,
        update_time: Option<&str>,
        token: &str,
    ) -> Result<(), String> {
        let body = json!({"fields": fields});
        let mut req = self.client.patch(path).bearer_auth(token).json(&body);

        // Attach precondition if we have an update_time from the last read
        if let Some(ut) = update_time {
            req = req.query(&[("currentDocument.updateTime", ut)]);
        }

        let resp = req.send().await.map_err(|e| format!("Firestore versioned PATCH failed: {e}"))?;

        match resp.status().as_u16() {
            200 | 201 => Ok(()),
            409 => Err("Optimistic concurrency conflict: document was modified concurrently".to_string()),
            status => {
                let body = resp.text().await.unwrap_or_default();
                Err(format!("Firestore PATCH {path} returned {status}: {body}"))
            }
        }
    }

    /// LIST documents in a collection with optional page size.
    async fn list_docs(&self, collection_url: &str, page_size: usize, token: &str) -> Result<Vec<Value>, String> {
        let resp = self
            .client
            .get(collection_url)
            .bearer_auth(token)
            .query(&[("pageSize", page_size.to_string().as_str())])
            .send()
            .await
            .map_err(|e| format!("Firestore LIST failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Firestore LIST {collection_url} returned {status}: {body}"));
        }

        let body: Value = resp.json().await.map_err(|e| e.to_string())?;
        let docs = body["documents"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        Ok(docs)
    }
}

// ─── CandidateRepository ──────────────────────────────────────────────────────

#[async_trait]
impl CandidateRepository for FirestoreStore {
    async fn create(&self, candidate: &CandidateRevision) -> Result<(), String> {
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(candidate.tenant_id, "candidates", &candidate.revision_id.0);
        let fields = to_firestore_fields(candidate)?;
        self.upsert_doc(&path, fields, &token).await?;
        info!("FirestoreStore: created candidate {}", candidate.revision_id.0);
        Ok(())
    }

    async fn get(&self, tenant_id: TenantId, revision_id: &RevisionId) -> Result<Option<CandidateRevision>, String> {
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(tenant_id, "candidates", &revision_id.0);
        match self.get_doc(&path, &token).await? {
            Some(doc) => Ok(Some(from_firestore_doc(&doc)?)),
            None => Ok(None),
        }
    }

    async fn list(&self, tenant_id: TenantId, limit: usize, _cursor: Option<String>) -> Result<Vec<CandidateRevision>, String> {
        let token = fetch_access_token(&self.client).await?;
        let col = self.collection_path(tenant_id, "candidates");
        let docs = self.list_docs(&col, limit, &token).await?;
        let mut results: Vec<CandidateRevision> = docs
            .iter()
            .filter_map(|d| from_firestore_doc(d).ok())
            .collect();
        results.sort_by(|a, b| b.abom.recorded_at.cmp(&a.abom.recorded_at));
        results.truncate(limit);
        Ok(results)
    }
}

// ─── WorkflowRepository ───────────────────────────────────────────────────────

#[async_trait]
impl WorkflowRepository for FirestoreStore {
    async fn create(&self, workflow: &Workflow) -> Result<(), String> {
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(workflow.tenant_id, "workflows", &workflow.workflow_id.to_string());
        let fields = to_firestore_fields(workflow)?;
        self.upsert_doc(&path, fields, &token).await?;
        info!("FirestoreStore: created workflow {}", workflow.workflow_id);
        Ok(())
    }

    async fn get(&self, tenant_id: TenantId, workflow_id: WorkflowId) -> Result<Option<Workflow>, String> {
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(tenant_id, "workflows", &workflow_id.to_string());
        match self.get_doc(&path, &token).await? {
            Some(doc) => Ok(Some(from_firestore_doc(&doc)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, workflow: &Workflow, expected_version: u64) -> Result<(), String> {
        // Verify the local version matches before writing.
        // We also attempt a Firestore precondition on update_time when available,
        // but the primary guard is our version field check.
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(workflow.tenant_id, "workflows", &workflow.workflow_id.to_string());

        // Read current version to validate optimistic concurrency
        if let Some(doc) = self.get_doc(&path, &token).await? {
            let current: Workflow = from_firestore_doc(&doc)?;
            if current.version != expected_version {
                return Err(format!(
                    "Optimistic concurrency failure: expected version {expected_version}, found {}",
                    current.version
                ));
            }
            // Extract Firestore update_time for precondition header
            let update_time = doc["updateTime"].as_str().map(str::to_string);
            let fields = to_firestore_fields(workflow)?;
            self.update_doc_versioned(&path, fields, update_time.as_deref(), &token).await?;
        } else {
            // Document doesn't exist yet — first write
            let fields = to_firestore_fields(workflow)?;
            self.upsert_doc(&path, fields, &token).await?;
        }

        info!(
            "FirestoreStore: updated workflow {} → state={:?} version={}",
            workflow.workflow_id, workflow.state, workflow.version
        );
        Ok(())
    }

    async fn list(
        &self,
        tenant_id: TenantId,
        state: Option<WorkflowState>,
        limit: usize,
        _cursor: Option<String>,
    ) -> Result<Vec<WorkflowSummary>, String> {
        let token = fetch_access_token(&self.client).await?;
        let col = self.collection_path(tenant_id, "workflows");
        let docs = self.list_docs(&col, limit.max(200), &token).await?;

        let mut results: Vec<WorkflowSummary> = docs
            .iter()
            .filter_map(|d| from_firestore_doc::<Workflow>(d).ok())
            .filter(|w| state.map_or(true, |s| w.state == s))
            .map(|w| w.to_summary())
            .collect();

        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results.truncate(limit);
        Ok(results)
    }
}

// ─── EvidenceRepository ───────────────────────────────────────────────────────

#[async_trait]
impl EvidenceRepository for FirestoreStore {
    async fn store_manifest(&self, manifest: &EvidenceManifestRef) -> Result<(), String> {
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(manifest.tenant_id, "evidence_manifests", &manifest.workflow_id.to_string());
        let fields = to_firestore_fields(manifest)?;
        self.upsert_doc(&path, fields, &token).await?;
        info!("FirestoreStore: stored evidence manifest for workflow {}", manifest.workflow_id);
        Ok(())
    }

    async fn get_manifest(&self, tenant_id: TenantId, workflow_id: WorkflowId) -> Result<Option<EvidenceManifestRef>, String> {
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(tenant_id, "evidence_manifests", &workflow_id.to_string());
        match self.get_doc(&path, &token).await? {
            Some(doc) => Ok(Some(from_firestore_doc(&doc)?)),
            None => Ok(None),
        }
    }
}

// ─── AuditRepository ──────────────────────────────────────────────────────────

#[async_trait]
impl AuditRepository for FirestoreStore {
    async fn append(&self, event: &AuditEvent) -> Result<(), String> {
        let token = fetch_access_token(&self.client).await?;
        // Use a random UUID as document ID — audit log is append-only.
        let doc_id = Uuid::new_v4().to_string();
        let path = self.doc_path(event.tenant_id, "audit_events", &doc_id);
        let fields = to_firestore_fields(event)?;
        self.upsert_doc(&path, fields, &token).await?;
        debug!("FirestoreStore: appended audit event {:?} for tenant {}", event.event_type, event.tenant_id);
        Ok(())
    }

    async fn query(&self, tenant_id: TenantId, workflow_id: Option<WorkflowId>, limit: usize) -> Result<Vec<AuditEvent>, String> {
        let token = fetch_access_token(&self.client).await?;
        let col = self.collection_path(tenant_id, "audit_events");
        let docs = self.list_docs(&col, limit.max(500), &token).await?;

        let mut results: Vec<AuditEvent> = docs
            .iter()
            .filter_map(|d| from_firestore_doc(d).ok())
            .filter(|e: &AuditEvent| {
                workflow_id.map_or(true, |wid| e.workflow_id == Some(wid))
            })
            .collect();

        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results.truncate(limit);
        Ok(results)
    }
}

// ─── FindingRepository ────────────────────────────────────────────────────────

#[async_trait]
impl FindingRepository for FirestoreStore {
    async fn store(&self, finding: &Finding) -> Result<(), String> {
        let token = fetch_access_token(&self.client).await?;
        let doc_id = Uuid::new_v4().to_string();
        let path = self.doc_path(finding.tenant_id, "findings", &doc_id);
        let fields = to_firestore_fields(finding)?;
        self.upsert_doc(&path, fields, &token).await?;
        debug!("FirestoreStore: stored finding for workflow {}", finding.workflow_id);
        Ok(())
    }

    async fn list(&self, tenant_id: TenantId, workflow_id: WorkflowId) -> Result<Vec<Finding>, String> {
        let token = fetch_access_token(&self.client).await?;
        let col = self.collection_path(tenant_id, "findings");
        let docs = self.list_docs(&col, 200, &token).await?;

        let results: Vec<Finding> = docs
            .iter()
            .filter_map(|d| from_firestore_doc(d).ok())
            .filter(|f: &Finding| f.workflow_id == workflow_id)
            .collect();

        Ok(results)
    }
}

// ─── ApprovalRepository ───────────────────────────────────────────────────────

#[async_trait]
impl ApprovalRepository for FirestoreStore {
    async fn create(&self, approval: &Approval) -> Result<(), String> {
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(approval.tenant_id, "approvals", &approval.approval_id.to_string());
        let fields = to_firestore_fields(approval)?;
        self.upsert_doc(&path, fields, &token).await?;
        info!("FirestoreStore: created approval {}", approval.approval_id);
        Ok(())
    }

    async fn get(&self, tenant_id: TenantId, approval_id: ApprovalId) -> Result<Option<Approval>, String> {
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(tenant_id, "approvals", &approval_id.to_string());
        match self.get_doc(&path, &token).await? {
            Some(doc) => Ok(Some(from_firestore_doc(&doc)?)),
            None => Ok(None),
        }
    }

    async fn get_for_workflow(&self, tenant_id: TenantId, workflow_id: WorkflowId) -> Result<Option<Approval>, String> {
        let token = fetch_access_token(&self.client).await?;
        let col = self.collection_path(tenant_id, "approvals");
        let docs = self.list_docs(&col, 50, &token).await?;

        let approval = docs
            .iter()
            .filter_map(|d| from_firestore_doc::<Approval>(d).ok())
            .find(|a| a.workflow_id == workflow_id);

        Ok(approval)
    }
}

// ─── AttestationRepository ────────────────────────────────────────────────────

#[async_trait]
impl AttestationRepository for FirestoreStore {
    async fn store(&self, workflow_id: WorkflowId, attestation: &AttestationMetadata) -> Result<(), String> {
        let token = fetch_access_token(&self.client).await?;
        let tenant_id = attestation.payload.tenant_id;
        let path = self.doc_path(tenant_id, "attestations", &workflow_id.to_string());
        let fields = to_firestore_fields(attestation)?;
        self.upsert_doc(&path, fields, &token).await?;
        info!("FirestoreStore: stored attestation for workflow {}", workflow_id);
        Ok(())
    }

    async fn get(&self, tenant_id: TenantId, workflow_id: WorkflowId) -> Result<Option<AttestationMetadata>, String> {
        let token = fetch_access_token(&self.client).await?;
        let path = self.doc_path(tenant_id, "attestations", &workflow_id.to_string());
        match self.get_doc(&path, &token).await? {
            Some(doc) => Ok(Some(from_firestore_doc(&doc)?)),
            None => Ok(None),
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Round-trip: Rust value → Firestore fields → Rust value.
    #[test]
    fn test_encode_decode_string() {
        let v = json!("hello world");
        let encoded = encode_value(&v);
        let decoded = decode_value(&encoded);
        assert_eq!(decoded, v);
    }

    #[test]
    fn test_encode_decode_integer() {
        let v = json!(42i64);
        let encoded = encode_value(&v);
        let decoded = decode_value(&encoded);
        assert_eq!(decoded, v);
    }

    #[test]
    fn test_encode_decode_bool() {
        let v = json!(true);
        let encoded = encode_value(&v);
        let decoded = decode_value(&encoded);
        assert_eq!(decoded, v);
    }

    #[test]
    fn test_encode_decode_array() {
        let v = json!(["a", "b", "c"]);
        let encoded = encode_value(&v);
        let decoded = decode_value(&encoded);
        assert_eq!(decoded, v);
    }

    #[test]
    fn test_encode_decode_nested_object() {
        let v = json!({"key": "value", "nested": {"num": 99}});
        let encoded = encode_value(&v);
        let decoded = decode_value(&encoded);
        assert_eq!(decoded, v);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct SampleDoc {
        id: String,
        count: i64,
        tags: Vec<String>,
    }

    #[test]
    fn test_roundtrip_struct() {
        let original = SampleDoc {
            id: "test-123".to_string(),
            count: 7,
            tags: vec!["a".to_string(), "b".to_string()],
        };
        let fields = to_firestore_fields(&original).unwrap();
        // Wrap in a fake document
        let doc = json!({"fields": fields});
        let decoded: SampleDoc = from_firestore_doc(&doc).unwrap();
        assert_eq!(decoded, original);
    }
}
