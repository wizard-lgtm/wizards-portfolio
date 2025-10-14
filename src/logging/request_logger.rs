use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub timestamp: DateTime<Utc>,
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
    pub timestamp: DateTime<Utc>,
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
    pub timestamp: DateTime<Utc>,
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