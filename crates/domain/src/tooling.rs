//! Tool and capability types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ids::RevisionId;

/// A tool action the candidate may invoke (e.g., "release_payment").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolAction {
    pub name: String,
    pub action: String,
}

impl ToolAction {
    pub fn new(name: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            action: action.into(),
        }
    }
}

/// Complete tool manifest for a candidate revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub schema_version: String,
    pub tools: Vec<ToolDefinition>,
    pub mcp_servers: Vec<McpServer>,
    pub digest: RevisionId,
}

/// Individual tool definition with strict schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value, // JSON Schema
    pub output_schema: serde_json::Value,
    pub required_permissions: Vec<String>,
    pub idempotent: bool,
    pub network_destinations: Vec<String>, // egress allowlist
    pub data_classification: Vec<String>,
}

/// MCP server reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub transport: McpTransport,
    pub endpoint: String,
    pub auth: McpAuth,
    pub capabilities: Vec<String>,
    pub digest: String, // server image/version digest
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    Http,
    Stdio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAuth {
    pub type_: String, // "workload_identity", "bearer", "mtls"
    pub config: HashMap<String, String>,
}

/// Effective permission granted to a candidate for a tool action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    pub tool_action: ToolAction,
    pub allowed: bool,
    pub conditions: Vec<PermissionCondition>,
}

/// Conditions under which a permission applies (time, resource, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCondition {
    pub kind: ConditionKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConditionKind {
    TimeWindow,
    ResourceTag,
    ApprovalId,
    Environment,
}
