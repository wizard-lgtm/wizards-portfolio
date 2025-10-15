pub mod health;
use actix_web::{web, Scope};
pub use health::health as health_handler;
pub fn api_scope() -> Scope {
    web::scope("/api")
        .service(health_handler)
}