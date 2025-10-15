use actix_web::{post, web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::logging::{request_logger::RequestLog, ClickLog, LoggerDb, RequestLogger};
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct ClickLogRequest {
    pub element: String,
    pub page_path: String,
    pub event_type: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestLogRequest {
    pub path: String,
    pub method: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub location: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

/// Log a click event
#[post("/click")]
pub async fn log_click(
    req: HttpRequest,
    body: web::Json<ClickLogRequest>,
    logger_db: web::Data<LoggerDb>,
) -> impl Responder {
    let ip = RequestLogger::extract_ip(&req);
    let user_agent = RequestLogger::extract_user_agent(&req);
    let request_id = RequestLogger::create_request_id();
    
    let click_log = ClickLog {
        id: None,
        timestamp: Utc::now(),
        request_id,
        ip_address: ip,
        user_agent,
        event_type: body.event_type.clone(),
        element: body.element.clone(),
        page_path: body.page_path.clone(),
    };

    match logger_db.log_click(click_log).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Click logged successfully".to_string(),
        }),
        Err(e) => {
            eprintln!("Failed to log click: {}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                message: "Failed to log click".to_string(),
            })
        }
    }
}

/// Log a request event
#[post("/request")]
pub async fn log_request(
    req: HttpRequest,
    body: web::Json<RequestLogRequest>,
    logger_db: web::Data<LoggerDb>,
) -> impl Responder {
    let ip = RequestLogger::extract_ip(&req);
    let user_agent = RequestLogger::extract_user_agent(&req);
    let request_id = RequestLogger::create_request_id();
    
    let request_log = RequestLog {
        id: None,
        timestamp: Utc::now(),
        request_id,
        ip_address: ip,
        user_agent,
        method: body.method.clone(),
        path: body.path.clone(),
        status_code: body.status_code,
        response_time_ms: body.response_time_ms,
        location: body.location.clone(),
        country: body.country.clone(),
        city: body.city.clone(),
    };

    match logger_db.log_request(request_log).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Request logged successfully".to_string(),
        }),
        Err(e) => {
            eprintln!("Failed to log request: {}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                message: "Failed to log request".to_string(),
            })
        }
    }
}