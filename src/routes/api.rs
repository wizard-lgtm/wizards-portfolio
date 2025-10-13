use actix_web::{get, web, Responder, HttpResponse};
use crate::db::MongoDb;

#[get("/health")]
pub async fn health(db: web::Data<MongoDb>) -> impl Responder {
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