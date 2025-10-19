use actix_files as fs;
use actix_web::dev::Service;
use actix_web::{middleware, web, App, HttpMessage, HttpServer};
use actix_web::{middleware::ErrorHandlers, http::StatusCode};
use actix_web::middleware::NormalizePath;
use dotenv::dotenv;
use std::env;

mod middlewares;
use middlewares::errors;
use errors::{internal_server_error_handler, not_found_handler};
use middlewares::request_logging::RequestLogging;
use crate::db::connect_with_retry;

mod config;
use config::{TEMPLATES, IS_DEV};

mod routes;
mod db;
mod types;
mod logging;

use routes::{pages_scope, api_scope, logs_scope};
use logging::{LoggerDb, RequestLogger};
use actix_web::middleware::Logger;

// -------------------- Server bootstrap --------------------

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load env
    dotenv().ok();
    
    // Setup logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Parse config
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid number");
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let env_mode = if *IS_DEV { "development" } else { "production" };
    
    println!("🚀 Starting server at http://{}:{}", host, port);
    println!("🔧 Environment: {}", env_mode);
    println!("📁 Serving templates from ./templates");
    println!("📦 Serving static files from ./static");
    
    if *IS_DEV {
        println!("⚠️  Development mode: Detailed errors will be shown");
        println!("💡 Set RUST_ENV=production to hide error details");
    }
    
    // Connect to MongoDB with retry logic
    println!("🔌 Connecting to MongoDB...");
    let mongodb = match connect_with_retry().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("❌ Failed to connect to MongoDB after retries: {}", e);
            std::process::exit(1);
        }
    };
    println!("✅ MongoDB connected successfully!");
    
    use std::sync::Arc;
    let mongodb = Arc::new(mongodb);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(LoggerDb::new(&mongodb)))
            .app_data(web::Data::from(mongodb.clone()))

            // Routes - organized by scope
            .service(api_scope())
            .service(logs_scope())
            .service(pages_scope())

            // Static files (CSS, JS, images, etc.)
            .service(fs::Files::new("/static", "./static").show_files_listing())
            
            // Middleware (order matters - applied in reverse order)
            .wrap(RequestLogging) // Our custom request logging
            .wrap(NormalizePath::trim())
            .wrap(Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::INTERNAL_SERVER_ERROR, internal_server_error_handler)
                    .handler(StatusCode::NOT_FOUND, not_found_handler)
            )
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}