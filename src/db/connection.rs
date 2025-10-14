use actix_web::web::get;
use mongodb::{Client, Database as MongoDatabase};
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use crate::types;

pub type Database = MongoDatabase;

pub fn get_database(client: &Client) -> Database {
    let db_name = env::var("MONGODB_DATABASE")
        .unwrap_or_else(|_| "wizards_portfolio".to_string());
    client.database(&db_name)
}

pub struct MongoDb {
    pub client: Client,
    pub database: MongoDatabase,
}

impl MongoDb {
    pub async fn new() -> mongodb::error::Result<Self> {
        let uri = env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let client = Client::with_uri_str(uri).await?;
        let database = get_database(&client);
        Ok(Self { client, database })
    }

    pub fn posts(&self) -> mongodb::Collection<types::Post> {
        self.database.collection("posts")
    }
}

/// Connect to MongoDB with exponential backoff retry logic
pub async fn connect_with_retry() -> Result<MongoDb, String> {
    let max_retries = 10;
    let mut retry_count = 0;
    let mut backoff_ms = 500;

    loop {
        match MongoDb::new().await {
            Ok(db) => {
                // Verify connection with a ping
                if let Err(e) = verify_connection(&db).await {
                    eprintln!("⚠️  MongoDB ping failed: {}", e);
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(format!(
                            "Failed to verify MongoDB connection after {} retries: {}",
                            max_retries, e
                        ));
                    }
                    println!(
                        "🔄 Retrying MongoDB connection in {}ms... (Attempt {}/{})",
                        backoff_ms, retry_count + 1, max_retries
                    );
                    sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms * 2).min(5000); // Max 5 seconds between retries
                    continue;
                }

                // Connection and ping successful
                return Ok(db);
            }
            Err(e) => {
                eprintln!("⚠️  MongoDB connection failed: {}", e);
                retry_count += 1;
                if retry_count >= max_retries {
                    return Err(format!(
                        "Failed to connect to MongoDB after {} retries: {}",
                        max_retries, e
                    ));
                }
                println!(
                    "🔄 Retrying MongoDB connection in {}ms... (Attempt {}/{})",
                    backoff_ms, retry_count + 1, max_retries
                );
                sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms = (backoff_ms * 2).min(5000); // Max 5 seconds between retries
            }
        }
    }
}

/// Verify MongoDB connection with a ping command
pub async fn verify_connection(db: &MongoDb) -> Result<(), String> {
    match db.database.run_command(
        mongodb::bson::doc! { "ping": 1 },
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Ping failed: {}", e)),
    }
}