// src/errors.rs
use actix_web::{dev::ServiceResponse, http::StatusCode, middleware::ErrorHandlerResponse, HttpResponse, Result, error};
use tera::Context;
use crate::{TEMPLATES, IS_DEV};

// ---------- Error Handlers ----------

pub fn internal_server_error_handler<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>>
where
    B: actix_web::body::MessageBody + 'static,
{
    let status = res.status();
    let real_error = res.response().error();
    
    // Get a cleaner message - extract the actual error string properly
    let error_message = if let Some(err) = real_error {
        err.to_string()
    } else {
        status.canonical_reason().unwrap_or("Unknown error").to_string()
    };
    
    eprintln!("Internal server error: {}", error_message);

    let (req, _res) = res.into_parts();

    // Detect if JSON should be returned instead of HTML
    let is_json_request = 
        req.path().starts_with("/api")
        || req.headers().get("Accept").map_or(false, |h| {
            h.to_str().map(|v| v.contains("application/json")).unwrap_or(false)
        });

    if is_json_request {
        let json_response = HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": status.canonical_reason().unwrap_or("Internal Server Error"),
            "details": if *IS_DEV { Some(error_message.as_str()) } else { None }
        }));

        return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
            req,
            json_response.map_into_right_body(),
        )));
    }

    let body = if *IS_DEV {
        let mut ctx = Context::new();
        ctx.insert("error_details", &error_message);

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
                    html_escape::encode_text(&error_message)
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

pub fn not_found_handler<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>>
where
    B: actix_web::body::MessageBody + 'static,
{
    let (req, _res) = res.into_parts();
    let path = req.path().to_string();

    // Detect if JSON should be returned instead of HTML
    let is_json_request = 
        req.path().starts_with("/api")
        || req.headers().get("Accept").map_or(false, |h| {
            h.to_str().map(|v| v.contains("application/json")).unwrap_or(false)
        });

    if is_json_request {
        let json_response = HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "Not Found",
            "details": if *IS_DEV { Some(path.as_str()) } else { None }
        }));

        return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
            req,
            json_response.map_into_right_body(),
        )));
    }

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
            Err(_) => {
                r#"<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body>
    <h1>404 Not Found</h1>
    <p>The page you're looking for doesn't exist.</p>
    <p><a href="/">Go Home</a></p>
</body>
</html>"#.to_string()
            }
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