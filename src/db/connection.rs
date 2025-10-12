use actix_web::web::get;
use mongodb::{Client, Database as MongoDatabase};
use std::env;

use crate::{db::posts, types};

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