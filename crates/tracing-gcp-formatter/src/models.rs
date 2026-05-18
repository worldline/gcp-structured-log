use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Google Structured Log Simplfied Format
// https://docs.cloud.google.com/logging/docs/structured-logging
#[allow(dead_code)]
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
pub struct SourceLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
}
