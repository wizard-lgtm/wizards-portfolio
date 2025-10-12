use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware};
use actix_files as fs;
use tera::{Tera, Context};
use lazy_static::lazy_static;
use std::env;
use dotenv::dotenv;
use actix_web::{middleware::ErrorHandlers, dev::ServiceResponse, http::StatusCode, Result, error};
use actix_web::middleware::ErrorHandlerResponse;

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

// Custom error handler for 500 errors
fn internal_server_error_handler<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> 
where
    B: actix_web::body::MessageBody + 'static,
{
    let error_msg = format!("{:?}", res.response());
    eprintln!("Internal server error: {}", error_msg);
    
    let (req, _res) = res.into_parts();
    
    // Show detailed error in development, generic message in production
    let body = if *IS_DEV {
        // In dev mode, try to render dev error template, fallback to inline HTML
        let mut ctx = Context::new();
        ctx.insert("error_details", &error_msg);
        
        match TEMPLATES.render("errors/500-dev.html", &ctx) {
            Ok(html) => html,
            Err(_) => {
                // Fallback inline HTML if template rendering fails
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
        // Production mode - render clean error page
        match TEMPLATES.render("errors/500.html", &Context::new()) {
            Ok(html) => html,
            Err(_) => {
                // Fallback inline HTML
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

// Custom error handler for 404 errors
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
        match TEMPLATES.render("404.html", &Context::new()) {
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

#[get("/")]
async fn index() -> Result<HttpResponse> {
    let mut ctx = Context::new();
    ctx.insert("name", "Wizards Portfolio");
    ctx.insert("title", "Home");
    
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
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment
    dotenv().ok();
    
    // Setup logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Parse configuration
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
    
    HttpServer::new(|| {
        App::new()
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
            // Favicon
            .service(fs::Files::new("/favicon.ico", "./static/favicon.ico"))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}