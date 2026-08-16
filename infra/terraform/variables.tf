# Chimera Sentinel — Terraform Variables

variable "project_id" {
  description = "GCP Project ID (e.g. sentinel-prod-123456)"
  type        = string
}

variable "region" {
  description = "GCP Region"
  type        = string
  default     = "us-east1"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}

variable "hot_path_min_instances" {
  description = "Warm instances for control-plane, ADK certifier, and mock ERP. Use 1 for recorded demos; keep 0 for lowest idle cost."
  type        = number
  default     = 0

  validation {
    condition     = contains([0, 1], var.hot_path_min_instances)
    error_message = "hot_path_min_instances must be 0 or 1."
  }
}

variable "web_url_override" {
  description = "Web console Cloud Run URL for the direct-browser CORS allowlist. Leave empty on first deploy, then set from the web_url output."
  type        = string
  default     = ""
}

variable "vpc_connector" {
  description = "VPC Connector resource name for private network access (optional, leave empty)"
  type        = string
  default     = ""
}

# ── Managed agent service resource names ─────────────────────────────────────
# These are populated AFTER running infra/scripts/setup-managed-agents.sh

# ── Firebase web app config ───────────────────────────────────────────────────
# These are public values (safe to store in tfvars — no secrets here).
variable "firebase_api_key" {
  description = "Firebase web app API key (public)"
  type        = string
  default     = "AIzaSyApjm0VdVcZyB8PZRBw99lv9MPx_m_EjTE"
}

variable "firebase_auth_domain" {
  description = "Firebase Auth domain"
  type        = string
  default     = "chimera-sentinel.firebaseapp.com"
}

variable "firebase_project_id" {
  description = "Firebase project ID"
  type        = string
  default     = "chimera-sentinel"
}

variable "firebase_storage_bucket" {
  description = "Firebase Storage bucket"
  type        = string
  default     = "chimera-sentinel.firebasestorage.app"
}

variable "firebase_messaging_sender_id" {
  description = "Firebase Messaging sender ID"
  type        = string
  default     = "817919872645"
}

variable "firebase_app_id" {
  description = "Firebase app ID"
  type        = string
  default     = "1:817919872645:web:b6826cca35d257c7e3e3ec"
}

variable "firebase_measurement_id" {
  description = "Firebase Analytics measurement ID"
  type        = string
  default     = "G-HP9JJWRTBS"
}

variable "gemini_model_ref" {
  description = "Batch case evaluation model — high volume, 80 cases per run"
  type        = string
  default     = "gemini-3.5-flash-lite"
}

variable "gemini_model_ref_agent" {
  description = "ADK agent orchestration model — certifier brain, tool-intent extraction"
  type        = string
  default     = "gemini-3.5-flash"
}

variable "gemini_model_ref_deep" {
  description = "Deep analysis model — ambiguous / escalated case reasoning"
  type        = string
  default     = "gemini-3.1-pro"
}

variable "model_armor_template" {
  description = "Model Armor template resource name"
  type        = string
  default     = ""
  # Example: projects/MY_PROJECT/locations/us-central1/templates/ap-agent-armor-v1
}

variable "agent_gateway_resource" {
  description = "Agent Gateway resource name"
  type        = string
  default     = ""
  # Example: projects/MY_PROJECT/locations/us-central1/gateways/ap-agent-gateway
}

variable "memory_bank_resource" {
  description = "Vertex AI RAG Corpus resource name (Memory Bank)"
  type        = string
  default     = ""
  # Example: projects/MY_PROJECT/locations/us-central1/ragCorpora/123456789
}
