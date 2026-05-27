use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

/// Google Structured Log Simplfied Format
// https://docs.cloud.google.com/logging/docs/structured-logging
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SimplifiedLogEntry<'a> {
    #[serde(default)]
    pub severity: Severity,
    pub message: Cow<'a, str>, // str will not work for escaped strings
    pub time: DateTime<Utc>,
    #[serde(rename = "httpRequest", skip_serializing_if = "Option::is_none")]
    pub http_request: Option<HttpRequest>,
    #[serde(
        rename = "logging.googleapis.com/operation",
        skip_serializing_if = "Option::is_none"
    )]
    pub operation: Option<LogEntryOperation>,
    #[serde(
        rename = "logging.googleapis.com/labels",
        skip_serializing_if = "Option::is_none"
    )]
    pub labels: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(
        rename = "logging.googleapis.com/sourceLocation",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_location: Option<SourceLocation>,
    #[serde(
        rename = "logging.googleapis.com/spanId",
        skip_serializing_if = "Option::is_none"
    )]
    pub span_id: Option<String>,
}

/// Google Structured Log Format
/// See https://docs.cloud.google.com/logging/docs/reference/v2/rest/v2/LogEntry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub log_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_request: Option<HttpRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<LogEntryOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_sampled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<Vec<SourceReference>>,
    // Payload fields (mutually exclusive in practice)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proto_payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<MonitoredResource>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "UPPERCASE"))]
pub enum Severity {
    #[default]
    #[serde(alias = "default", alias = "DEFAULT")]
    Default,
    #[serde(alias = "debug", alias = "DEBUG")]
    Debug,
    #[serde(alias = "info", alias = "INFO")]
    Info,
    #[serde(alias = "notice", alias = "NOTICE")]
    Notice,
    #[serde(alias = "warning", alias = "WARNING")]
    Warning,
    #[serde(alias = "error", alias = "ERROR")]
    Error,
    #[serde(alias = "critical", alias = "CRITICAL")]
    Critical,
    #[serde(alias = "alert", alias = "ALERT")]
    Alert,
    #[serde(alias = "emergency", alias = "EMERGENCY")]
    Emergency,
}

//TODO Should be in CLI crate only
impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Default => write!(f, "TRACE"),
            Severity::Debug => write!(f, "DEBUG"),
            Severity::Info => write!(f, " INFO"),
            Severity::Notice => write!(f, "NOTICE"),
            Severity::Warning => write!(f, " WARN"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, " CRIT"),
            Severity::Alert => write!(f, "ALERT"),
            Severity::Emergency => write!(f, "EMERG"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_validated_with_origin_server: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntryOperation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceReference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoredResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
}
