use actix_web::{get, web, Responder, HttpResponse};
use crate::db::MongoDb;

#[get("/health")]
pub async fn health(db: web::Data<MongoDb>) -> impl Responder {
    match db.database.run_command(
        mongodb::bson::doc! { "ping": 1 },
    ).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "database": "connected"
        })),
        Err(err) => {
            eprintln!("Health ping failed: {}", err);

            HttpResponse::Ok().json(serde_json::json!({ // !TODO, for now we can't properly handle 500 response code due to generic internal server eroor handler failure. Fix it
                "status": "unhealthy",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "database": "disconnected",
                "error": err.to_string()
            }))
        }
    }
}