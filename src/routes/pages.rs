
use actix_web::{error, get, web, HttpResponse, Result, Scope};
use tera::Context;
use crate::{db, errors, };
use crate::db::MongoDb;
use crate::types::PostStatus;
use crate::errors::{internal_server_error_handler, not_found_handler};
use crate::TEMPLATES;

// -------------------- Handlers --------------------

#[get("/")]
pub async fn index(db: web::Data<MongoDb>) -> Result<HttpResponse> {
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
pub async fn about() -> Result<HttpResponse> {
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


pub fn pages_scope() -> Scope {
    web::scope("")
        .service(index)
        .service(about)
}