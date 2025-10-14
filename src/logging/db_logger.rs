use mongodb::bson::doc;
use crate::db::MongoDb;
use crate::logging::request_logger::RequestLog;
use crate::logging::{RequestLogger, ClickLog, SystemPerformanceLog};

pub struct LoggerDb;

impl LoggerDb {
    pub fn log_request_collection(mongo_db: &MongoDb) -> mongodb::Collection<RequestLog> {
        mongo_db.database.collection::<RequestLog>("request_logs")
    }

    pub fn log_click_collection(mongo_db: &MongoDb) -> mongodb::Collection<ClickLog> {
        mongo_db.database.collection::<ClickLog>("click_logs")
    }

    pub fn log_performance_collection(mongo_db: &MongoDb) -> mongodb::Collection<SystemPerformanceLog> {
        mongo_db.database.collection::<SystemPerformanceLog>("performance_logs")
    }

    pub async fn log_request(
        mongo_db: &MongoDb,
        log: RequestLog,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = Self::log_request_collection(mongo_db);
        collection.insert_one(&log).await?;
        Ok(())
    }

    pub async fn log_click(
        mongo_db: &MongoDb,
        log: ClickLog,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = Self::log_click_collection(mongo_db);
        collection.insert_one(&log).await?;
        Ok(())
    }

    pub async fn log_performance(
        mongo_db: &MongoDb,
        log: SystemPerformanceLog,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = Self::log_performance_collection(mongo_db);
        collection.insert_one(&log).await?;
        Ok(())
    }

    pub async fn get_request_stats_by_date(
        mongo_db: &MongoDb,
        date: &str, // Format: "2024-01-15"
    ) -> Result<mongodb::bson::Document, Box<dyn std::error::Error>> {
        let collection = Self::log_request_collection(mongo_db);
        
        let pipeline = vec![
            doc! {
                "$match": {
                    "timestamp": {
                        "$regex": format!("^{}", date),
                        "$options": "i"
                    }
                }
            },
            doc! {
                "$group": {
                    "_id": None,
                    "total_requests": { "$sum": 1 },
                    "avg_response_time": { "$avg": "$response_time_ms" },
                    "unique_ips": { "$addToSet": "$ip_address" },
                }
            },
        ];

        let mut cursor = collection.aggregate(pipeline).await?;
        
        if let Some(doc) = cursor.next().await {
            Ok(doc?)
        } else {
            Ok(doc! {})
        }
    }

    pub async fn get_click_stats(
        mongo_db: &MongoDb,
        ip: Option<&str>,
    ) -> Result<Vec<mongodb::bson::Document>, Box<dyn std::error::Error>> {
        let collection = Self::log_click_collection(mongo_db);
        
        let filter = match ip {
            Some(ip_addr) => doc! { "ip_address": ip_addr },
            None => doc! {},
        };

        let pipeline = vec![
            doc! { "$match": filter },
            doc! {
                "$group": {
                    "_id": "$element",
                    "click_count": { "$sum": 1 },
                    "unique_users": { "$addToSet": "$ip_address" }
                }
            },
            doc! { "$sort": { "click_count": -1 } },
        ];

        let mut cursor = collection.aggregate(pipeline).await?;
        let mut results = Vec::new();
        
        while let Some(doc) = cursor.next().await {
            results.push(doc?);
        }

        Ok(results)
    }

    pub async fn get_unique_ips(
        mongo_db: &MongoDb,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let collection = Self::log_request_collection(mongo_db);
        
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": "$ip_address"
                }
            },
        ];

        let mut cursor = collection.aggregate(pipeline).await?;
        let mut ips = Vec::new();
        
        while let Some(doc) = cursor.next().await {
            if let Ok(doc) = doc {
                if let Some(ip) = doc.get_str("_id").ok() {
                    ips.push(ip.to_string());
                }
            }
        }

        Ok(ips)
    }

    pub async fn get_requests_by_ip(
        mongo_db: &MongoDb,
        ip: &str,
    ) -> Result<Vec<RequestLog>, Box<dyn std::error::Error>> {
        let collection = Self::log_request_collection(mongo_db);
        let filter = doc! { "ip_address": ip };
        
        let mut cursor = collection.find(filter).await?;
        let mut results = Vec::new();
        
        while let Some(doc) = cursor.next().await {
            results.push(doc?);
        }

        Ok(results)
    }

    pub async fn get_total_requests_by_day(
        mongo_db: &MongoDb,
    ) -> Result<Vec<mongodb::bson::Document>, Box<dyn std::error::Error>> {
        let collection = Self::log_request_collection(mongo_db);
        
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": {
                        "$dateToString": {
                            "format": "%Y-%m-%d",
                            "date": "$timestamp"
                        }
                    },
                    "total_requests": { "$sum": 1 },
                    "avg_response_time": { "$avg": "$response_time_ms" }
                }
            },
            doc! { "$sort": { "_id": 1 } },
        ];

        let mut cursor = collection.aggregate(pipeline).await?;
        let mut results = Vec::new();
        
        while let Some(doc) = cursor.next().await {
            results.push(doc?);
        }

        Ok(results)
    }

    pub async fn get_total_clicks_by_day(
        mongo_db: &MongoDb,
    ) -> Result<Vec<mongodb::bson::Document>, Box<dyn std::error::Error>> {
        let collection = Self::log_click_collection(mongo_db);
        
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": {
                        "$dateToString": {
                            "format": "%Y-%m-%d",
                            "date": "$timestamp"
                        }
                    },
                    "total_clicks": { "$sum": 1 }
                }
            },
            doc! { "$sort": { "_id": 1 } },
        ];

        let mut cursor = collection.aggregate(pipeline).await?;
        let mut results = Vec::new();
        
        while let Some(doc) = cursor.next().await {
            results.push(doc?);
        }

        Ok(results)
    }
}