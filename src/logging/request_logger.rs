use serde::{Deserialize, Deserializer, Serialize};
use mongodb::bson::oid::ObjectId;

// Custom deserializer to handle both string and DateTime formats
fn deserialize_flexible_datetime<'de, D>(deserializer: D) -> Result<mongodb::bson::DateTime, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FlexibleDateTime {
        DateTime(mongodb::bson::DateTime),
        String(String),
    }
    
    match FlexibleDateTime::deserialize(deserializer)? {
        FlexibleDateTime::DateTime(dt) => Ok(dt),
        FlexibleDateTime::String(s) => {
            // Try to parse the string as an ISO 8601 datetime
            use chrono::{DateTime, Utc};
            let parsed: DateTime<Utc> = s.parse()
                .map_err(|e| Error::custom(format!("Failed to parse datetime string: {}", e)))?;
            Ok(mongodb::bson::DateTime::from_millis(parsed.timestamp_millis()))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(deserialize_with = "deserialize_flexible_datetime")]
    pub timestamp: mongodb::bson::DateTime,
    pub request_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub location: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(deserialize_with = "deserialize_flexible_datetime")]
    pub timestamp: mongodb::bson::DateTime,
    pub request_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub event_type: String,
    pub element: String,
    pub page_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformanceLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(deserialize_with = "deserialize_flexible_datetime")]
    pub timestamp: mongodb::bson::DateTime,
    pub render_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub active_connections: usize,
    pub request_queue_depth: usize,
}

pub struct RequestLogger;

impl RequestLogger {
    pub fn create_request_id() -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }

    pub fn extract_ip(req: &actix_web::HttpRequest) -> String {
        req.connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string()
    }

    pub fn extract_user_agent(req: &actix_web::HttpRequest) -> String {
        req.headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string()
    }

    pub fn extract_path(req: &actix_web::HttpRequest) -> String {
        req.path().to_string()
    }

    pub fn extract_method(req: &actix_web::HttpRequest) -> String {
        req.method().to_string()
    }
}