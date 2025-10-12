use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::env;
use dotenv::dotenv; 

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Welcome to wizards-portfolio!")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load .env file

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    println!("Application running on port {}", port);
    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}