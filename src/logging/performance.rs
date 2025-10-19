// src/logging/performance.rs
use std::time::Instant;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use mongodb::bson::DateTime as BsonDateTime;
use super::SystemPerformanceLog;

#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    start_time: Instant,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        PerformanceTracker {
            start_time: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
    pub fn create_system_log(
        render_time_ms: f64,
        active_connections: usize,
        request_queue_depth: usize,
    ) -> SystemPerformanceLog {
        SystemPerformanceLog {
            id: None,
            timestamp: BsonDateTime::from_millis(Utc::now().timestamp_millis()),
            render_time_ms,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            active_connections,
            request_queue_depth,
        }
    }
}


impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}