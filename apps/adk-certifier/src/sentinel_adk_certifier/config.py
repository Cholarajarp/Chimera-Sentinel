"""Configuration for ADK Certifier.

Three Gemini models are used for different roles:
  GEMINI_MODEL_REF       — batch case evaluation  (gemini-3.5-flash-lite, high volume, GA)
  GEMINI_MODEL_REF_AGENT — ADK agent orchestration (gemini-3.5-flash, agentic workloads, GA)
  GEMINI_MODEL_REF_DEEP  — deep analysis / Pro     (gemini-3.1-pro-preview, ambiguous escalation)
"""

from pydantic import Field
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    host: str = Field(default="0.0.0.0", validation_alias="SENTINEL_ADK_HOST")
    port: int = Field(default=8081, validation_alias="SENTINEL_ADK_PORT")
    google_cloud_project: str = Field(default="", validation_alias="GOOGLE_CLOUD_PROJECT")
    google_cloud_region: str = Field(default="us-east1", validation_alias="GOOGLE_CLOUD_REGION")

    # Batch evaluation model — high volume, 80 cases per run (cheap & fast, GA)
    gemini_model_ref: str = Field(
        default="gemini-3.5-flash-lite", validation_alias="GEMINI_MODEL_REF"
    )
    # ADK agent orchestration model — certifier brain, tool-intent extraction (GA)
    gemini_model_ref_agent: str = Field(
        default="gemini-3.5-flash", validation_alias="GEMINI_MODEL_REF_AGENT"
    )
    # Deep analysis model — ambiguous / escalated case reasoning (Preview)
    gemini_model_ref_deep: str = Field(
        default="gemini-3.1-pro-preview", validation_alias="GEMINI_MODEL_REF_DEEP"
    )

    # Model Armor template resource name
    # e.g. projects/{project}/locations/{region}/modelArmorTemplates/{id}
    model_armor_template: str = Field(default="", validation_alias="MODEL_ARMOR_TEMPLATE")
    # Agent Gateway resource name
    # e.g. projects/{project}/locations/{region}/gateways/{id}
    gateway_resource: str = Field(default="", validation_alias="AGENT_GATEWAY_RESOURCE")
    # Memory Bank resource name
    # e.g. projects/{project}/locations/{region}/ragCorpora/{id}
    memory_bank_resource: str = Field(default="", validation_alias="MEMORY_BANK_RESOURCE")
    # Internal URL of the Enterprise ERP Adapter MCP service
    erp_mcp_url: str = Field(default="http://localhost:9090", validation_alias="ERP_MCP_URL")
    queue_transport: str = Field(default="in-memory", validation_alias="QUEUE_TRANSPORT")
    log_level: str = Field(default="INFO", validation_alias="LOG_LEVEL")

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"
        case_sensitive = False
