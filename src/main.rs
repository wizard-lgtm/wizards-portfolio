use actix_files as fs;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware};
use actix_web::{middleware::ErrorHandlers, dev::ServiceResponse, http::StatusCode, Result, error};
use actix_web::middleware::ErrorHandlerResponse;
use dotenv::dotenv;
use lazy_static::lazy_static;
use std::env;

use tera::{Tera, Context};

use crate::db::MongoDb;
use crate::types::PostStatus;
use actix_web::HttpRequest;

/// Re-export db module (ensure db/mod.rs `pub use connection::MongoDb;` exists)
mod db;
mod types;

lazy_static! {
    static ref TEMPLATES: Tera = {
        match Tera::new("src/templates/**/*") {
            Ok(mut tera) => {
                tera.autoescape_on(vec!["html", "htm"]);
                tera
            }
            Err(e) => {
                eprintln!("Template parsing error: {}", e);
                std::process::exit(1);
            }
        }
    };

    static ref IS_DEV: bool = {
        env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "development"
    };
}

// ---------- Error Handlers (kept similar to your previous handlers) ----------

fn internal_server_error_handler<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>>
where
    B: actix_web::body::MessageBody + 'static,
{
    let error_msg = format!("{:?}", res.response());
    eprintln!("Internal server error: {}", error_msg);

    let (req, _res) = res.into_parts();

    let body = if *IS_DEV {
        let mut ctx = Context::new();
        ctx.insert("error_details", &error_msg);

        match TEMPLATES.render("errors/500-dev.html", &ctx) {
            Ok(html) => html,
            Err(_) => {
                format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>500 Internal Server Error</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; 
               max-width: 900px; margin: 50px auto; padding: 20px; line-height: 1.6; }}
        h1 {{ color: #d32f2f; }}
        .error-box {{ background: #f5f5f5; border-left: 4px solid #d32f2f; 
                      padding: 15px; margin: 20px 0; overflow-x: auto; }}
        pre {{ margin: 0; white-space: pre-wrap; word-wrap: break-word; }}
        .tip {{ background: #fff3cd; border-left: 4px solid #ffc107; 
                padding: 15px; margin: 20px 0; }}
    </style>
</head>
<body>
    <h1>🔥 500 Internal Server Error</h1>
    <p>An error occurred while processing your request.</p>
    
    <div class="error-box">
        <strong>Error Details:</strong>
        <pre>{}</pre>
    </div>
    
    <div class="tip">
        <strong>💡 Debugging Tips:</strong>
        <ul>
            <li>Check that your templates directory exists and contains the required .html files</li>
            <li>Verify template syntax is correct</li>
            <li>Check server logs for more details</li>
            <li>Set <code>RUST_ENV=production</code> to hide detailed errors</li>
        </ul>
    </div>
</body>
</html>"#,
                    html_escape::encode_text(&error_msg)
                )
            }
        }
    } else {
        match TEMPLATES.render("errors/500.html", &Context::new()) {
            Ok(html) => html,
            Err(_) => {
                r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>500 Internal Server Error</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
               max-width: 600px; margin: 100px auto; padding: 20px; text-align: center; }
        h1 { color: #d32f2f; font-size: 3em; }
        a { color: #4a90e2; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <h1>500</h1>
    <h2>Internal Server Error</h2>
    <p>Something went wrong on our end. Please try again later.</p>
    <p><a href="/">← Back to Home</a></p>
</body>
</html>"#.to_string()
            }
        }
    };

    let new_response = HttpResponse::InternalServerError()
        .content_type("text/html; charset=utf-8")
        .body(body);

    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        req,
        new_response.map_into_right_body(),
    )))
}

fn not_found_handler<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>>
where
    B: actix_web::body::MessageBody + 'static,
{
    let (req, _res) = res.into_parts();
    let path = req.path().to_string();

    let body = if *IS_DEV {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>404 Not Found</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; 
               max-width: 800px; margin: 50px auto; padding: 20px; text-align: center; }}
        h1 {{ color: #ff6b6b; font-size: 4em; margin: 0; }}
        .path {{ background: #f5f5f5; padding: 10px; border-radius: 5px; 
                 font-family: monospace; margin: 20px 0; }}
        a {{ color: #4a90e2; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>404</h1>
    <h2>Page Not Found</h2>
    <p>The requested path was not found:</p>
    <div class="path">{}</div>
    <p><a href="/">← Back to Home</a></p>
</body>
</html>"#,
            html_escape::encode_text(&path)
        )
    } else {
        match TEMPLATES.render("errors/404.html", &Context::new()) {
            Ok(html) => html,
            Err(_) => format!(
                r#"<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body>
    <h1>404 Not Found</h1>
    <p>The page you're looking for doesn't exist.</p>
    <p><a href="/">Go Home</a></p>
</body>
</html>"#
            )
        }
    };

    let new_response = HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(body);

    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        req,
        new_response.map_into_right_body(),
    )))
}

// -------------------- Handlers --------------------

#[get("/")]
async fn index(db: web::Data<MongoDb>) -> Result<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("name", "Wizards Portfolio");
    ctx.insert("title", "Home");

    // Fetch latest 5 published posts
    let posts = match db::posts::list_posts(&db.database, Some(PostStatus::Published), 5, 0).await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Error fetching posts: {}", e);
            Vec::new()
        }
    };

    ctx.insert("posts", &posts);

    let rendered = TEMPLATES.render("index.html", &ctx)
        .map_err(|e| {
            eprintln!("Template rendering error: {}", e);
            error::ErrorInternalServerError("Template rendering failed")
        })?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered))
}

#[get("/about")]
async fn about() -> Result<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("title", "About");

    let rendered = TEMPLATES.render("about.html", &ctx)
        .map_err(|e| {
            eprintln!("Template rendering error: {}", e);
            error::ErrorInternalServerError("Template rendering failed")
        })?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered))
}

#[get("/health")]
async fn health(db: web::Data<MongoDb>) -> impl Responder {
    // Try to ping database
    let db_status = match db.database.run_command(
        mongodb::bson::doc! { "ping": 1 },
    ).await {
        Ok(_) => "connected",
        Err(err) => {
            eprintln!("Health ping failed: {}", err);
            "disconnected"
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "database": db_status
    }))
}

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
            .service(index)
            .service(about)
            .service(health)
            // Static files (CSS, JS, images, etc.)
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // Favicon route

    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
