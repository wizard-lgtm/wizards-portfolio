use actix_files as fs;
use actix_web::dev::Service;
use actix_web::{get, middleware, web, App, HttpMessage, HttpResponse, HttpServer, Responder};
use actix_web::{middleware::ErrorHandlers, dev::ServiceResponse, http::StatusCode, Result, error};
use actix_web::middleware::ErrorHandlerResponse;
use dotenv::dotenv;
use lazy_static::lazy_static;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

mod errors;
use errors::{internal_server_error_handler, not_found_handler};
use tera::{Tera, Context};
use crate::db::{connect_with_retry, MongoDb};
use crate::types::PostStatus;
mod config;
use config::{TEMPLATES, IS_DEV};
/// Re-export db module (ensure db/mod.rs `pub use connection::MongoDb;` exists)
mod routes;
mod db;
mod types;
use routes::{pages, api};
mod logging;
use logging::{LoggerDb, RequestLogger, PerformanceTracker};
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
            .wrap(Logger::default())
            // Share database connection
            .app_data(web::Data::from(mongodb.clone()))
            // Middleware
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::INTERNAL_SERVER_ERROR, internal_server_error_handler)
                    .handler(StatusCode::NOT_FOUND, not_found_handler)
            )
            .wrap_fn(|req, srv| {
            let request_id = RequestLogger::create_request_id();
            let start = std::time::Instant::now();
            
            req.extensions_mut().insert(request_id.clone());
            
            let fut = srv.call(req);
            
            async move {
                let res = fut.await?;
                let elapsed = start.elapsed().as_millis() as u64;
                
                // Log the response with timing
                Ok(res)
            }
        })
            // Routes
            .service(pages::index)
            .service(pages::about)
            .service(api::api_scope())
            // Static files (CSS, JS, images, etc.)
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // Favicon route
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
