# ── Chimera Sentinel Terraform Outputs ───────────────────────────────────────

output "control_plane_url" {
  description = "Control plane REST API URL — used as the web service API_URL and SENTINEL_API_URL for smoke tests"
  value       = google_cloud_run_v2_service.control_plane.uri
}

output "workflow_worker_job" {
  description = "One-shot workflow worker Cloud Run Job name"
  value       = google_cloud_run_v2_job.workflow_worker.name
}

output "adk_certifier_url" {
  description = "ADK Certifier internal URL"
  value       = google_cloud_run_v2_service.adk_certifier.uri
}

output "mock_erp_url" {
  description = "Mock ERP MCP URL"
  value       = google_cloud_run_v2_service.mock_erp.uri
}

output "web_url" {
  description = "Web console public URL — open this in the browser for the demo"
  value       = google_cloud_run_v2_service.web.uri
}

output "artifact_registry" {
  description = "Artifact Registry repository path"
  value       = "${var.region}-docker.pkg.dev/${var.project_id}/sentinel"
}

output "firestore_database" {
  description = "Firestore database name"
  value       = google_firestore_database.sentinel.name
}

output "evidence_bucket" {
  description = "GCS bucket for evidence bundles"
  value       = google_storage_bucket.evidence.name
}

output "kms_key" {
  description = "Cloud KMS attestation signing key resource name"
  value       = google_kms_crypto_key.attestation_signer.id
}

output "kms_key_ring" {
  description = "Cloud KMS key ring resource name"
  value       = google_kms_key_ring.sentinel.id
}
