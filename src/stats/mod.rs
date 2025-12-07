use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    pub total_requests: usize,
    pub total_duration: f64,
    pub error_count: usize,
    pub status_codes: HashMap<u16, usize>,
    pub sql_queries: usize,
    pub total_sql_duration: f64,
}

impl PerformanceStats {
    pub fn avg_response_time(&self) -> f64 {
        if self.total_requests > 0 {
            self.total_duration / self.total_requests as f64
        } else {
            0.0
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests > 0 {
            (self.error_count as f64 / self.total_requests as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn avg_sql_time(&self) -> f64 {
        if self.sql_queries > 0 {
            self.total_sql_duration / self.sql_queries as f64
        } else {
            0.0
        }
    }
}

#[derive(Clone)]
pub struct StatsCollector {
    stats: Arc<Mutex<PerformanceStats>>,
}

impl StatsCollector {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(PerformanceStats::default())),
        }
    }

    pub fn record_request(&self, status: u16, duration: f64) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_requests += 1;
        stats.total_duration += duration;

        if status >= 400 {
            stats.error_count += 1;
        }

        *stats.status_codes.entry(status).or_insert(0) += 1;
    }

    pub fn record_sql_query(&self, duration: f64) {
        let mut stats = self.stats.lock().unwrap();
        stats.sql_queries += 1;
        stats.total_sql_duration += duration;
    }

    pub fn get_stats(&self) -> PerformanceStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn reset(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = PerformanceStats::default();
    }
}
