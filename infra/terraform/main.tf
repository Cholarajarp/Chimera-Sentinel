# Chimera Sentinel — GCP Infrastructure
# Deploys the complete Sentinel stack to Google Cloud Run
#
# Usage:
#   cp infra/terraform/terraform.tfvars.example infra/terraform/terraform.tfvars
#   # edit terraform.tfvars with your project_id
#   terraform -chdir=infra/terraform init
#   terraform -chdir=infra/terraform apply

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "~> 5.0"
    }
  }

  # GCS backend — bucket must exist before first init.
  # Bootstrap: gcloud storage buckets create gs://chimera-sentinel-tf-state --project=chimera-sentinel --location=us-east1
  backend "gcs" {
    bucket = "chimera-sentinel-tf-state"
    prefix = "sentinel/prod"
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

provider "google-beta" {
  project = var.project_id
  region  = var.region
}

# ── Enable required APIs ──────────────────────────────────────────────────────
resource "google_project_service" "apis" {
  for_each = toset([
    "run.googleapis.com",
    "cloudbuild.googleapis.com",
    "artifactregistry.googleapis.com",
    "firestore.googleapis.com",
    "storage.googleapis.com",
    "cloudkms.googleapis.com",
    "secretmanager.googleapis.com",
    "monitoring.googleapis.com",
    "logging.googleapis.com",
    "cloudtrace.googleapis.com",
    "iam.googleapis.com",
    "iamcredentials.googleapis.com",
    "aiplatform.googleapis.com",
  ])

  service            = each.key
  disable_on_destroy = false
}

# ── Artifact Registry ─────────────────────────────────────────────────────────
resource "google_artifact_registry_repository" "sentinel" {
  location      = var.region
  repository_id = "sentinel"
  format        = "DOCKER"
  description   = "Docker images for Sentinel services"

  cleanup_policies {
    id     = "keep-recent-images"
    action = "KEEP"

    most_recent_versions {
      keep_count = 5
    }
  }

  depends_on = [google_project_service.apis["artifactregistry.googleapis.com"]]
}

# ── Firestore ─────────────────────────────────────────────────────────────────
resource "google_firestore_database" "sentinel" {
  project     = var.project_id
  name        = "(default)"
  location_id = var.region
  type        = "FIRESTORE_NATIVE"

  depends_on = [google_project_service.apis["firestore.googleapis.com"]]
}

# ── Cloud Storage (evidence bundle archive) ───────────────────────────────────
resource "google_storage_bucket" "evidence" {
  name                        = "${var.project_id}-sentinel-evidence"
  location                    = var.region
  uniform_bucket_level_access = true

  versioning {
    enabled = true
  }

  lifecycle_rule {
    condition { age = 365 }
    action { type = "Delete" }
  }
}

# ── Cloud KMS key ring + asymmetric signing key ───────────────────────────────
resource "google_kms_key_ring" "sentinel" {
  name     = "sentinel-ring"
  location = "global"

  depends_on = [google_project_service.apis["cloudkms.googleapis.com"]]
}

resource "google_kms_crypto_key" "attestation_signer" {
  name     = "attestation-signer"
  key_ring = google_kms_key_ring.sentinel.id
  purpose  = "ASYMMETRIC_SIGN"

  version_template {
    algorithm        = "RSA_SIGN_PSS_2048_SHA256"
    protection_level = "SOFTWARE"
  }

  # no rotation_period for asymmetric keys — manual rotation only
  lifecycle {
    prevent_destroy = true
  }
}

# ── Secret Manager ────────────────────────────────────────────────────────────
resource "google_secret_manager_secret" "sentinel_config" {
  secret_id = "sentinel-config"
  replication {
    auto {}
  }

  depends_on = [google_project_service.apis["secretmanager.googleapis.com"]]
}

resource "google_secret_manager_secret_version" "sentinel_config" {
  secret = google_secret_manager_secret.sentinel_config.id
  secret_data = templatefile("${path.module}/config/sentinel-prod.toml", {
    project_id  = var.project_id
    region      = var.region
    environment = var.environment
  })
}

# ── Service Accounts ──────────────────────────────────────────────────────────
resource "google_service_account" "control_plane" {
  account_id   = "sentinel-control-plane"
  display_name = "Sentinel Control Plane"
}

resource "google_service_account" "workflow_worker" {
  account_id   = "sentinel-workflow-worker"
  display_name = "Sentinel Workflow Worker"
}

resource "google_service_account" "adk_certifier" {
  account_id   = "sentinel-adk-certifier"
  display_name = "Sentinel ADK Certifier"
}

resource "google_service_account" "mock_erp" {
  account_id   = "sentinel-mock-erp"
  display_name = "Sentinel Enterprise ERP Adapter MCP"
}

resource "google_service_account" "web" {
  account_id   = "sentinel-web"
  display_name = "Sentinel Web Console"
}

# ── IAM bindings ──────────────────────────────────────────────────────────────
# Keys are static identifiers; service-account emails are apply-time values and
# must stay out of for_each keys or planning fails before anything is created.
locals {
  sa_roles = {
    control_plane = {
      member = "serviceAccount:${google_service_account.control_plane.email}"
      roles = [
        "roles/datastore.user",
        "roles/storage.objectUser",
        "roles/cloudkms.signer",
        "roles/cloudkms.signerVerifier",
        "roles/secretmanager.secretAccessor",
      ]
    }
    workflow_worker = {
      member = "serviceAccount:${google_service_account.workflow_worker.email}"
      roles = [
        "roles/datastore.user",
        "roles/storage.objectUser",
        "roles/cloudkms.signer",
        "roles/cloudkms.signerVerifier",
        "roles/secretmanager.secretAccessor",
      ]
    }
    adk_certifier = {
      member = "serviceAccount:${google_service_account.adk_certifier.email}"
      roles = [
        "roles/aiplatform.user",
        "roles/secretmanager.secretAccessor",
      ]
    }
  }
}

resource "google_project_iam_member" "sa_bindings" {
  for_each = merge([
    for name, cfg in local.sa_roles : {
      for role in cfg.roles : "${name}__${role}" => { member = cfg.member, role = role }
    }
  ]...)

  project = var.project_id
  role    = each.value.role
  member  = each.value.member
}

# KMS key-level signer grant (required for asymmetric sign)
resource "google_kms_crypto_key_iam_member" "worker_kms_signer" {
  crypto_key_id = google_kms_crypto_key.attestation_signer.id
  role          = "roles/cloudkms.signerVerifier"
  member        = "serviceAccount:${google_service_account.workflow_worker.email}"
}

resource "google_kms_crypto_key_iam_member" "control_plane_kms_signer" {
  crypto_key_id = google_kms_crypto_key.attestation_signer.id
  role          = "roles/cloudkms.signerVerifier"
  member        = "serviceAccount:${google_service_account.control_plane.email}"
}

# ── Cloud Run: Control Plane ──────────────────────────────────────────────────
resource "google_cloud_run_v2_service" "control_plane" {
  name     = "sentinel-control-plane"
  location = var.region

  template {
    service_account = google_service_account.control_plane.email

    scaling {
      min_instance_count = var.hot_path_min_instances
      max_instance_count = 1
    }

    containers {
      image = "${var.region}-docker.pkg.dev/${var.project_id}/sentinel/control-plane:latest"

      ports {
        container_port = 8080
      }

      resources {
        limits   = { cpu = "1", memory = "512Mi" }
        cpu_idle = true
      }

      env {
        name  = "GOOGLE_CLOUD_PROJECT"
        value = var.project_id
      }
      env {
        name  = "GOOGLE_CLOUD_REGION"
        value = var.region
      }
      env {
        name  = "SENTINEL_WORKFLOW_JOB"
        value = "sentinel-workflow-worker"
      }
      env {
        name  = "SENTINEL_DATABASE__TYPE"
        value = "firestore"
      }
      env {
        name  = "SENTINEL_DATABASE__PROJECT_ID"
        value = var.project_id
      }
      env {
        name  = "SENTINEL_DATABASE__DATABASE_ID"
        value = "(default)"
      }
      env {
        name  = "SENTINEL_ENVIRONMENT"
        value = var.environment
      }
      env {
        name  = "SENTINEL_SERVER__ADDR"
        value = "0.0.0.0:8080"
      }
      env {
        name  = "SENTINEL_GOOGLE_CLOUD__USE_LIVE_SERVICES"
        value = "true"
      }
      env {
        name  = "SENTINEL_GOOGLE_CLOUD__KMS_KEY_RESOURCE"
        value = "projects/${var.project_id}/locations/global/keyRings/sentinel-ring/cryptoKeys/attestation-signer"
      }
      env {
        name  = "SENTINEL_GOOGLE_CLOUD__KMS_KEY_VERSION"
        value = "1"
      }
      env {
        name  = "SENTINEL_GOOGLE_CLOUD__STORAGE_BUCKET"
        value = google_storage_bucket.evidence.name
      }
      env {
        name  = "SENTINEL_ALLOWED_ORIGIN"
        value = var.web_url_override
      }
    }

    execution_environment = "EXECUTION_ENVIRONMENT_GEN2"
  }

  traffic {
    percent = 100
    type    = "TRAFFIC_TARGET_ALLOCATION_TYPE_LATEST"
  }

  depends_on = [
    google_firestore_database.sentinel,
    google_project_service.apis["run.googleapis.com"],
  ]
}

resource "google_cloud_run_v2_service_iam_member" "control_plane_public" {
  name     = google_cloud_run_v2_service.control_plane.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "allUsers"
}

# ── Cloud Run Job: Workflow Worker ───────────────────────────────────────────
resource "google_cloud_run_v2_job" "workflow_worker" {
  name     = "sentinel-workflow-worker"
  location = var.region

  template {
    task_count  = 1
    parallelism = 1

    template {
      service_account = google_service_account.workflow_worker.email
      timeout         = "1800s"
      max_retries     = 1

      containers {
        image = "${var.region}-docker.pkg.dev/${var.project_id}/sentinel/workflow-worker:latest"
        args  = ["--once"]

        resources {
          limits = { cpu = "1", memory = "1Gi" }
        }

        env {
          name  = "GOOGLE_CLOUD_PROJECT"
          value = var.project_id
        }
        env {
          name  = "SENTINEL_DATABASE__TYPE"
          value = "firestore"
        }
        env {
          name  = "SENTINEL_DATABASE__PROJECT_ID"
          value = var.project_id
        }
        env {
          name  = "SENTINEL_DATABASE__DATABASE_ID"
          value = "(default)"
        }
        env {
          name  = "SENTINEL_GOOGLE_CLOUD__USE_LIVE_SERVICES"
          value = "true"
        }
        env {
          name  = "SENTINEL_GOOGLE_CLOUD__KMS_KEY_RESOURCE"
          value = google_kms_crypto_key.attestation_signer.id
        }
        env {
          name  = "SENTINEL_GOOGLE_CLOUD__KMS_KEY_VERSION"
          value = "1"
        }
        env {
          name  = "SENTINEL_ADK_CERTIFIER_URL"
          value = google_cloud_run_v2_service.adk_certifier.uri
        }
      }
    }
  }

  depends_on = [
    google_firestore_database.sentinel,
    google_cloud_run_v2_service.adk_certifier,
  ]
}

resource "google_cloud_run_v2_job_iam_member" "control_plane_worker_invoker" {
  name     = google_cloud_run_v2_job.workflow_worker.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "serviceAccount:${google_service_account.control_plane.email}"
}

# ── Cloud Run: ADK Certifier ──────────────────────────────────────────────────
resource "google_cloud_run_v2_service" "adk_certifier" {
  name     = "sentinel-adk-certifier"
  location = var.region

  template {
    service_account = google_service_account.adk_certifier.email
    timeout         = "1800s"

    scaling {
      min_instance_count = var.hot_path_min_instances
      max_instance_count = 1
    }

    containers {
      image = "${var.region}-docker.pkg.dev/${var.project_id}/sentinel/adk-certifier:latest"

      ports {
        container_port = 8081
      }

      resources {
        limits   = { cpu = "1", memory = "1Gi" }
        cpu_idle = true
      }

      env {
        name  = "GOOGLE_CLOUD_PROJECT"
        value = var.project_id
      }
      env {
        name  = "GOOGLE_CLOUD_REGION"
        value = var.region
      }
      env {
        name  = "GEMINI_MODEL_REF"
        value = var.gemini_model_ref
      }
      env {
        name  = "GEMINI_MODEL_REF_AGENT"
        value = var.gemini_model_ref_agent
      }
      env {
        name  = "GEMINI_MODEL_REF_DEEP"
        value = var.gemini_model_ref_deep
      }
      env {
        name  = "MODEL_ARMOR_TEMPLATE"
        value = var.model_armor_template
      }
      env {
        name  = "AGENT_GATEWAY_RESOURCE"
        value = var.agent_gateway_resource
      }
      env {
        name  = "MEMORY_BANK_RESOURCE"
        value = var.memory_bank_resource
      }
      env {
        name  = "ERP_MCP_URL"
        value = google_cloud_run_v2_service.mock_erp.uri
      }
      env {
        name  = "SENTINEL_ADK_PORT"
        value = "8081"
      }
    }

    execution_environment = "EXECUTION_ENVIRONMENT_GEN2"
  }

  traffic {
    percent = 100
    type    = "TRAFFIC_TARGET_ALLOCATION_TYPE_LATEST"
  }

  depends_on = [google_project_service.apis["run.googleapis.com"]]
}

resource "google_cloud_run_v2_service_iam_member" "adk_certifier_internal" {
  name     = google_cloud_run_v2_service.adk_certifier.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "serviceAccount:${google_service_account.workflow_worker.email}"
}

# ── Cloud Run: Enterprise ERP Adapter MCP ───────────────────────────────────────────────────
resource "google_cloud_run_v2_service" "mock_erp" {
  name     = "sentinel-enterprise-erp-adapter"
  location = var.region

  template {
    service_account = google_service_account.mock_erp.email

    scaling {
      min_instance_count = var.hot_path_min_instances
      max_instance_count = 1
    }

    containers {
      image = "${var.region}-docker.pkg.dev/${var.project_id}/sentinel/enterprise-erp-adapter:latest"

      ports {
        container_port = 9090
      }

      resources {
        limits   = { cpu = "1", memory = "512Mi" }
        cpu_idle = true
      }

      env {
        name  = "GOOGLE_CLOUD_PROJECT"
        value = var.project_id
      }
    }

    execution_environment = "EXECUTION_ENVIRONMENT_GEN2"
  }

  traffic {
    percent = 100
    type    = "TRAFFIC_TARGET_ALLOCATION_TYPE_LATEST"
  }

  depends_on = [google_project_service.apis["run.googleapis.com"]]
}

resource "google_cloud_run_v2_service_iam_member" "mock_erp_internal" {
  name     = google_cloud_run_v2_service.mock_erp.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "serviceAccount:${google_service_account.adk_certifier.email}"
}

# ── Cloud Run: Web Console ────────────────────────────────────────────────────
resource "google_cloud_run_v2_service" "web" {
  name     = "sentinel-web"
  location = var.region

  template {
    service_account = google_service_account.web.email

    scaling {
      # hot_path_min_instances defaults to 0 (zero idle cost).
      # Set to 1 only when presenting a live demo to avoid cold-start latency.
      min_instance_count = var.hot_path_min_instances
      max_instance_count = 1
    }

    containers {
      image = "${var.region}-docker.pkg.dev/${var.project_id}/sentinel/web:latest"

      ports {
        container_port = 3000
      }

      resources {
        limits   = { cpu = "1", memory = "512Mi" }
        cpu_idle = true
      }

      env {
        name  = "NODE_ENV"
        value = "production"
      }
      env {
        name  = "API_URL"
        value = google_cloud_run_v2_service.control_plane.uri
      }
      env {
        name  = "NEXT_PUBLIC_SITE_URL"
        value = "https://chimera-sentinel.web.app"
      }
      # Firebase public config — baked at build time (see Dockerfile ARGs)
      # but also surfaced here so runtime server-side code can read them.
      env {
        name  = "NEXT_PUBLIC_FIREBASE_API_KEY"
        value = var.firebase_api_key
      }
      env {
        name  = "NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN"
        value = var.firebase_auth_domain
      }
      env {
        name  = "NEXT_PUBLIC_FIREBASE_PROJECT_ID"
        value = var.firebase_project_id
      }
      env {
        name  = "NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET"
        value = var.firebase_storage_bucket
      }
      env {
        name  = "NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID"
        value = var.firebase_messaging_sender_id
      }
      env {
        name  = "NEXT_PUBLIC_FIREBASE_APP_ID"
        value = var.firebase_app_id
      }
      env {
        name  = "NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID"
        value = var.firebase_measurement_id
      }
    }

    execution_environment = "EXECUTION_ENVIRONMENT_GEN2"
  }

  traffic {
    percent = 100
    type    = "TRAFFIC_TARGET_ALLOCATION_TYPE_LATEST"
  }

  depends_on = [google_project_service.apis["run.googleapis.com"]]
}

resource "google_cloud_run_v2_service_iam_member" "web_public" {
  name     = google_cloud_run_v2_service.web.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "allUsers"
}
