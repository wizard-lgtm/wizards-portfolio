use actix_web::{get, web, HttpResponse, Responder, Scope};
use crate::config::{TEMPLATES, IS_DEV};
use crate::logging::LoggerDb;
use tera::Context;

/// View all request logs with pagination
#[get("/requests")]
pub async fn view_requests(
    logger_db: web::Data<LoggerDb>,
) -> impl Responder {
    match logger_db.get_total_requests_by_day().await {
        Ok(logs) => {
            let mut context = Context::new();
            context.insert("logs", &logs);
            context.insert("page_title", "Request Logs");
            
            match TEMPLATES.render("logs/requests.html", &context) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => {
                    eprintln!("Template error: {}", e);
                    let error_message = if *IS_DEV {
                        format!("Template error: {}", e)
                    } else {
                        "Template rendering error".to_string()
                    };
                    HttpResponse::InternalServerError().body(error_message)
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let error_message = if *IS_DEV {
                format!("Database error: {}", e)
            } else {
                "Failed to fetch logs".to_string()
            };
            HttpResponse::InternalServerError().body(error_message)
        }
    }
}

/// View click logs
#[get("/clicks")]
pub async fn view_clicks(
    logger_db: web::Data<LoggerDb>,
) -> impl Responder {
    match logger_db.get_click_stats(None).await {
        Ok(stats) => {
            let mut context = Context::new();
            context.insert("stats", &stats);
            context.insert("page_title", "Click Statistics");
            
            match TEMPLATES.render("logs/clicks.html", &context) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => {
                    eprintln!("Template error: {}", e);
                    let error_message = if *IS_DEV {
                        format!("Template error: {}", e)
                    } else {
                        "Template rendering error".to_string()
                    };
                    HttpResponse::InternalServerError().body(error_message)
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let error_message = if *IS_DEV {
                format!("Database error: {}", e)
            } else {
                "Failed to fetch click stats".to_string()
            };
            HttpResponse::InternalServerError().body(error_message)
        }
    }
}

/// View logs dashboard
#[get("")]
pub async fn logs_dashboard(
    logger_db: web::Data<LoggerDb>,
) -> impl Responder {
    // Fetch summary data
    let requests_by_day = logger_db.get_total_requests_by_day().await.ok();
    let clicks_by_day = logger_db.get_total_clicks_by_day().await.ok();
    let unique_ips = logger_db.get_unique_ips().await.ok();
    
    let mut context = Context::new();
    context.insert("page_title", "Logs Dashboard");
    context.insert("requests_by_day", &requests_by_day);
    context.insert("clicks_by_day", &clicks_by_day);
    
    if let Some(ips) = &unique_ips {
        context.insert("unique_ip_count", &ips.len());
    }
    
    match TEMPLATES.render("logs/dashboard.html", &context) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => {
            eprintln!("Template error: {}", e);
            HttpResponse::InternalServerError().body("Template rendering error")
        }
    }
}

/// View logs by specific IP
#[get("/ip/{ip}")]
pub async fn view_logs_by_ip(
    ip: web::Path<String>,
    logger_db: web::Data<LoggerDb>,
) -> impl Responder {
    let ip = ip.into_inner();
    
    match logger_db.get_requests_by_ip(&ip).await {
        Ok(logs) => {
            let mut context = Context::new();
            context.insert("logs", &logs);
            context.insert("ip_address", &ip);
            context.insert("page_title", &format!("Logs for IP: {}", ip));
            
            match TEMPLATES.render("logs/ip_logs.html", &context) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => {
                    eprintln!("Template error: {}", e);
                    let error_message = if *IS_DEV {
                        format!("Template error: {}", e)
                    } else {
                        "Template rendering error".to_string()
                    };
                    HttpResponse::InternalServerError().body(error_message)
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let error_message = if *IS_DEV {
                format!("Database error: {}", e)
            } else {
                "Failed to fetch logs".to_string()
            };
            HttpResponse::InternalServerError().body(error_message)
        }
    }
}

/// View logs for a specific date
#[get("/date/{date}")]
pub async fn view_logs_by_date(
    date: web::Path<String>,
    logger_db: web::Data<LoggerDb>,
) -> impl Responder {
    let date = date.into_inner();
    
    match logger_db.get_request_stats_by_date(&date).await {
        Ok(stats) => {
            let mut context = Context::new();
            context.insert("stats", &stats);
            context.insert("date", &date);
            context.insert("page_title", &format!("Logs for {}", date));
            
            match TEMPLATES.render("logs/date_stats.html", &context) {
                Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
                Err(e) => {
                    eprintln!("Template error: {}", e);
                    let error_message = if *IS_DEV {
                        format!("Template error: {}", e)
                    } else {
                        "Template rendering error".to_string()
                    };
                    HttpResponse::InternalServerError().body(error_message)
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let error_message = if *IS_DEV {
                format!("Database error: {}", e)
            } else {
                "Failed to fetch stats".to_string()
            };
            HttpResponse::InternalServerError().body(error_message)
        }
    }
}

// -------------------- Scope --------------------

pub fn logs_scope() -> Scope {
    web::scope("/logs")
        .service(logs_dashboard)
        .service(view_requests)
        .service(view_clicks)
        .service(view_logs_by_ip)
        .service(view_logs_by_date)
}