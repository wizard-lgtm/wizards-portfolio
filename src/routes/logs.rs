use actix_web::{get, post, delete, web, HttpResponse, Responder, Scope};
use serde::Deserialize;
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
    let total_requests = logger_db.get_total_request_count().await.unwrap_or(0);
    let total_clicks = logger_db.get_total_click_count().await.unwrap_or(0);
    
    let mut context = Context::new();
    context.insert("page_title", "Logs Dashboard");
    context.insert("requests_by_day", &requests_by_day);
    context.insert("clicks_by_day", &clicks_by_day);
    context.insert("total_requests", &total_requests);
    context.insert("total_clicks", &total_clicks);
    
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

/// View all individual requests with pagination and search
#[get("/requests/all")]
pub async fn view_all_requests(
    logger_db: web::Data<LoggerDb>,
    query: web::Query<RequestsQuery>,
) -> impl Responder {
    let search_query = query.search.as_deref();
    let page = query.page.unwrap_or(1);
    let per_page = 50;
    let skip = ((page - 1) * per_page) as u64;
    
    let logs_result = if let Some(search) = search_query {
        logger_db.search_requests(search).await
    } else {
        logger_db.get_all_requests(Some(per_page as i64), Some(skip)).await
    };
    
    match logs_result {
        Ok(logs) => {
            let total_count = logger_db.get_total_request_count().await.unwrap_or(0);
            let total_pages = (total_count as f64 / per_page as f64).ceil() as u64;
            
            let mut context = Context::new();
            context.insert("logs", &logs);
            context.insert("page_title", "All Request Logs");
            context.insert("current_page", &page);
            context.insert("total_pages", &total_pages);
            context.insert("total_count", &total_count);
            context.insert("search_query", &search_query);
            
            match TEMPLATES.render("logs/all_requests.html", &context) {
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

#[derive(Deserialize)]
pub struct RequestsQuery {
    pub search: Option<String>,
    pub page: Option<u64>,
}

/// Delete logs by date
#[delete("/date/{date}")]
pub async fn delete_logs_by_date(
    date: web::Path<String>,
    logger_db: web::Data<LoggerDb>,
) -> impl Responder {
    let date = date.into_inner();
    
    match logger_db.delete_requests_by_date(&date).await {
        Ok(deleted_count) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "deleted_count": deleted_count,
                "message": format!("Deleted {} requests from {}", deleted_count, date)
            }))
        }
        Err(e) => {
            eprintln!("Failed to delete logs: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Failed to delete logs"
            }))
        }
    }
}

/// View detailed logs for a specific date
#[get("/date/{date}/details")]
pub async fn view_date_details(
    date: web::Path<String>,
    logger_db: web::Data<LoggerDb>,
) -> impl Responder {
    let date = date.into_inner();
    
    match logger_db.get_requests_by_date(&date).await {
        Ok(logs) => {
            let mut context = Context::new();
            context.insert("logs", &logs);
            context.insert("date", &date);
            context.insert("page_title", &format!("Request Details for {}", date));
            context.insert("total_count", &logs.len());
            
            match TEMPLATES.render("logs/date_details.html", &context) {
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

// -------------------- Scope --------------------

pub fn logs_scope() -> Scope {
    web::scope("/logs")
        .service(logs_dashboard)
        .service(view_requests)
        .service(view_all_requests)
        .service(view_clicks)
        .service(view_logs_by_ip)
        .service(view_logs_by_date)
        .service(view_date_details)
        .service(delete_logs_by_date)
}