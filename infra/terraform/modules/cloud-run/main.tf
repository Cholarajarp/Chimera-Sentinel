# Cloud Run Service Module

variable "service_name" {
  description = "Name of the Cloud Run service"
  type        = string
}

variable "image" {
  description = "Docker image URL"
  type        = string
}

variable "service_account" {
  description = "Service account email"
  type        = string
}

variable "region" {
  description = "GCP Region"
  type        = string
}

variable "port" {
  description = "Container port"
  type        = number
  default     = 8080
}

variable "cpu" {
  description = "CPU allocation"
  type        = string
  default     = "1"
}

variable "memory" {
  description = "Memory allocation"
  type        = string
  default     = "1Gi"
}

variable "min_instances" {
  description = "Minimum number of instances"
  type        = number
  default     = 0
}

variable "max_instances" {
  description = "Maximum number of instances"
  type        = number
  default     = 10
}

variable "env_vars" {
  description = "Environment variables"
  type        = map(string)
  default     = {}
}

variable "secret_env_vars" {
  description = "Secret environment variables from Secret Manager"
  type        = map(string)
  default     = {}
}

variable "vpc_connector" {
  description = "VPC Connector for private network access"
  type        = string
  default     = ""
}

variable "concurrency" {
  description = "Max concurrent requests per instance"
  type        = number
  default     = 80
}

variable "timeout_seconds" {
  description = "Request timeout in seconds"
  type        = number
  default     = 300
}

# Cloud Run service
resource "google_cloud_run_v2_service" "service" {
  name     = var.service_name
  location = var.region

  template {
    service_account = var.service_account

    containers {
      image = var.image
      ports {
        name          = "http1"
        container_port = var.port
      }
      resources {
        limits = {
          cpu    = var.cpu
          memory = var.memory
        }
      }
      env {
        for_each = var.env_vars
        name     = each.key
        value    = each.value
      }
      env {
        for_each = var.secret_env_vars
        name     = each.key
        value_source {
          secret_key_ref {
            secret = each.value
            version = "latest"
          }
        }
      }
    }

    scaling {
      min_instance_count = var.min_instances
      max_instance_count = var.max_instances
    }

    vpc_access {
      egress = var.vpc_connector != "" ? "PRIVATE_RANGES_ONLY" : "ALL_TRAFFIC"
      network_interfaces {
        network = var.vpc_connector != "" ? "projects/${google_cloud_run_v2_service.service.project}/global/networks/default" : null
      }
    }

    execution_environment = "EXECUTION_ENVIRONMENT_GEN2"
  }

  traffic {
    percent         = 100
    latest_revision = true
  }
}

# Allow unauthenticated invocations (for public endpoints)
resource "google_cloud_run_v2_service_iam_member" "public" {
  service  = google_cloud_run_v2_service.service.name
  location = var.region
  role     = "roles/run.invoker"
  member   = "allUsers"
}

# Cloud Run service URL output
output "service_url" {
  value = google_cloud_run_v2_service.service.uri
}

output "service_name" {
  value = google_cloud_run_v2_service.service.name
}