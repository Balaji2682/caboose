use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_requests: usize,
    pub total_duration: f64,
    pub error_count: usize,
    pub status_codes: HashMap<u16, usize>,
    pub sql_queries: usize,
    pub total_sql_duration: f64,
    pub response_time_history: Vec<u64>, // History of average response times
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            total_duration: 0.0,
            error_count: 0,
            status_codes: HashMap::new(),
            sql_queries: 0,
            total_sql_duration: 0.0,
            response_time_history: Vec::with_capacity(100), // Pre-allocate capacity
        }
    }
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

        // Update response time history (rolling average)
        let current_avg = stats.avg_response_time().round() as u64;
        stats.response_time_history.push(current_avg);
        if stats.response_time_history.len() > 100 {
            stats.response_time_history.remove(0); // Keep history to last 100 entries
        }
    }

    pub fn record_sql_query(&self, duration: f64) {
        let mut stats = self.stats.lock().unwrap();
        stats.sql_queries += 1;
        stats.total_sql_duration += duration;
    }

    pub fn get_stats(&self) -> PerformanceStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn get_response_time_history(&self) -> Vec<u64> {
        self.stats.lock().unwrap().response_time_history.clone()
    }

    pub fn reset(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = PerformanceStats::default();
    }
}
