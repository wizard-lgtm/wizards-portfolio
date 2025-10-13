use actix_files as fs;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware};
use actix_web::{middleware::ErrorHandlers, dev::ServiceResponse, http::StatusCode, Result, error};
use actix_web::middleware::ErrorHandlerResponse;
use dotenv::dotenv;
use lazy_static::lazy_static;
use std::env;
mod errors;
use errors::{internal_server_error_handler, not_found_handler};

use tera::{Tera, Context};

use crate::db::MongoDb;
use crate::types::PostStatus;

mod config;
use config::{TEMPLATES, IS_DEV};

/// Re-export db module (ensure db/mod.rs `pub use connection::MongoDb;` exists)
mod routes;
mod db;
mod types;

use routes::{pages, api};


// -------------------- Server bootstrap --------------------

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load env
    dotenv().ok();

    // Setup logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Connect to MongoDB
    let mongodb = match MongoDb::new().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to connect to MongoDB: {}", e);
            std::process::exit(1);
        }
    };

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

    use std::sync::Arc;
    let mongodb = Arc::new(match MongoDb::new().await {
        Ok(m) => m,
        Err(e) => { eprintln!("Failed to connect to MongoDB: {}", e); std::process::exit(1); }
    });
    HttpServer::new(move || {
        
        App::new()
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
            // Routes
            .service(pages::index)
            .service(pages::about)
            .service(api::health)
            // Static files (CSS, JS, images, etc.)
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // Favicon route

    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
