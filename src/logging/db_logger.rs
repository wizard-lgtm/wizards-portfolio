use mongodb::bson::doc;
use futures::StreamExt;
use std::sync::Arc;
use crate::db::MongoDb;
use crate::logging::request_logger::RequestLog;
use crate::logging::{RequestLogger, ClickLog, SystemPerformanceLog};
use futures::TryStreamExt;

pub struct LoggerDb {
    mongo_db: Arc<MongoDb>,
}

impl LoggerDb {
    pub fn new(mongo_db: &Arc<MongoDb>) -> Self {
        Self {
            mongo_db: Arc::clone(mongo_db),
        }
    }

    fn log_request_collection(&self) -> mongodb::Collection<RequestLog> {
        self.mongo_db.database.collection::<RequestLog>("request_logs")
    }

    fn log_click_collection(&self) -> mongodb::Collection<ClickLog> {
        self.mongo_db.database.collection::<ClickLog>("click_logs")
    }

    fn log_performance_collection(&self) -> mongodb::Collection<SystemPerformanceLog> {
        self.mongo_db.database.collection::<SystemPerformanceLog>("performance_logs")
    }

    pub async fn log_request(
        &self,
        log: RequestLog,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.log_request_collection();
        collection.insert_one(&log).await?;
        Ok(())
    }

    pub async fn log_click(
        &self,
        log: ClickLog,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.log_click_collection();
        collection.insert_one(&log).await?;
        Ok(())
    }

    pub async fn log_performance(
        &self,
        log: SystemPerformanceLog,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.log_performance_collection();
        collection.insert_one(&log).await?;
        Ok(())
    }

    pub async fn get_request_stats_by_date(
        &self,
        date: &str, // Format: "2024-01-15"
    ) -> Result<mongodb::bson::Document, Box<dyn std::error::Error>> {
        let collection = self.log_request_collection();
        
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
                    "_id": mongodb::bson::Bson::Null,
                    "total_requests": { "$sum": 1 },
                    "avg_response_time": { "$avg": "$response_time_ms" },
                    "unique_ips": { "$addToSet": "$ip_address" },
                }
            },
        ];

        let mut cursor = collection.aggregate(pipeline).await?;
        
        if let Some(result) = cursor.try_next().await? {
            Ok(result)
        } else {
            Ok(doc! {})
        }
    }

    pub async fn get_click_stats(
        &self,
        ip: Option<&str>,
    ) -> Result<Vec<mongodb::bson::Document>, Box<dyn std::error::Error>> {
        let collection = self.log_click_collection();
        
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
        
        while let Some(result) = cursor.try_next().await? {
            results.push(result);
        }

        Ok(results)
    }

    pub async fn get_unique_ips(
        &self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let collection = self.log_request_collection();
        
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": "$ip_address"
                }
            },
        ];

        let mut cursor = collection.aggregate(pipeline).await?;
        let mut ips = Vec::new();
        
        while let Some(result) = cursor.try_next().await? {
            if let Some(ip) = result.get_str("_id").ok() {
                ips.push(ip.to_string());
            }
        }

        Ok(ips)
    }

    pub async fn get_requests_by_ip(
        &self,
        ip: &str,
    ) -> Result<Vec<RequestLog>, Box<dyn std::error::Error>> {
        let collection = self.log_request_collection();
        let filter = doc! { "ip_address": ip };
        
        let mut cursor = collection.find(filter).await?;
        let mut results = Vec::new();
        
        while let Some(result) = cursor.try_next().await? {
            results.push(result);
        }

        Ok(results)
    }

    pub async fn get_total_requests_by_day(
        &self,
    ) -> Result<Vec<mongodb::bson::Document>, Box<dyn std::error::Error>> {
        let collection = self.log_request_collection();
        
        let pipeline = vec![
            doc! {
                "$addFields": {
                    "timestamp_date": {
                        "$cond": [
                            { "$eq": [{ "$type": "$timestamp" }, "date"] },
                            "$timestamp",
                            { "$dateFromString": { "dateString": "$timestamp" } }
                        ]
                    }
                }
            },
            doc! {
                "$group": {
                    "_id": {
                        "$dateToString": {
                            "format": "%Y-%m-%d",
                            "date": "$timestamp_date"
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
        
        while let Some(result) = cursor.try_next().await? {
            results.push(result);
        }

        Ok(results)
    }

    pub async fn get_total_clicks_by_day(
        &self,
    ) -> Result<Vec<mongodb::bson::Document>, Box<dyn std::error::Error>> {
        let collection = self.log_click_collection();
        
        let pipeline = vec![
            doc! {
                "$addFields": {
                    "timestamp_date": {
                        "$cond": [
                            { "$eq": [{ "$type": "$timestamp" }, "date"] },
                            "$timestamp",
                            { "$dateFromString": { "dateString": "$timestamp" } }
                        ]
                    }
                }
            },
            doc! {
                "$group": {
                    "_id": {
                        "$dateToString": {
                            "format": "%Y-%m-%d",
                            "date": "$timestamp_date"
                        }
                    },
                    "total_clicks": { "$sum": 1 }
                }
            },
            doc! { "$sort": { "_id": 1 } },
        ];

        let mut cursor = collection.aggregate(pipeline).await?;
        let mut results = Vec::new();
        
        while let Some(result) = cursor.try_next().await? {
            results.push(result);
        }

        Ok(results)
    }
}