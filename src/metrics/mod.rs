use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::System;

// Memory management constants
const MAX_ENDPOINTS: usize = 500;
const ENDPOINTS_WARNING_THRESHOLD: usize = 450; // 90% of max

/// Time-series data point
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub timestamp: Instant,
    pub value: f64,
}

/// Time-series storage with automatic cleanup
#[derive(Debug, Clone)]
pub struct TimeSeries {
    data: VecDeque<DataPoint>,
    max_age: Duration,
    max_points: usize,
}

impl TimeSeries {
    pub fn new(max_age: Duration, max_points: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(max_points),
            max_age,
            max_points,
        }
    }

    pub fn add(&mut self, value: f64) {
        let now = Instant::now();

        // Remove old data points
        while let Some(point) = self.data.front() {
            if now.duration_since(point.timestamp) > self.max_age {
                self.data.pop_front();
            } else {
                break;
            }
        }

        // Add new point
        self.data.push_back(DataPoint {
            timestamp: now,
            value,
        });

        // Limit total points
        while self.data.len() > self.max_points {
            self.data.pop_front();
        }
    }

    pub fn get_recent(&self, duration: Duration) -> Vec<DataPoint> {
        let now = Instant::now();
        self.data
            .iter()
            .filter(|p| now.duration_since(p.timestamp) <= duration)
            .cloned()
            .collect()
    }

    pub fn get_all(&self) -> Vec<DataPoint> {
        self.data.iter().cloned().collect()
    }

    pub fn average(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.data.iter().map(|p| p.value).sum();
        sum / self.data.len() as f64
    }

    pub fn percentile(&self, p: f64) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }

        let mut values: Vec<f64> = self.data.iter().map(|p| p.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((p / 100.0) * values.len() as f64) as usize;
        values[index.min(values.len() - 1)]
    }

    pub fn max(&self) -> f64 {
        self.data.iter().map(|p| p.value).fold(0.0, f64::max)
    }

    pub fn min(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min)
    }
}

/// Response time statistics per endpoint
#[derive(Debug, Clone)]
pub struct EndpointStats {
    pub path: String,
    pub count: usize,
    pub total_duration: f64,
    pub min_duration: f64,
    pub max_duration: f64,
    pub durations: Vec<f64>, // Keep last N durations for percentile calc
}

impl EndpointStats {
    pub fn new(path: String) -> Self {
        Self {
            path,
            count: 0,
            total_duration: 0.0,
            min_duration: f64::INFINITY,
            max_duration: 0.0,
            durations: Vec::new(),
        }
    }

    pub fn add_request(&mut self, duration: f64) {
        self.count += 1;
        self.total_duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);

        self.durations.push(duration);
        // Keep only last 1000 durations
        if self.durations.len() > 1000 {
            self.durations.remove(0);
        }
    }

    pub fn avg_duration(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_duration / self.count as f64
        }
    }

    pub fn percentile(&self, p: f64) -> f64 {
        if self.durations.is_empty() {
            return 0.0;
        }

        let mut sorted = self.durations.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((p / 100.0) * sorted.len() as f64) as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

/// Advanced metrics collector with real-time monitoring
pub struct AdvancedMetrics {
    // Time-series data
    request_rate: Arc<Mutex<TimeSeries>>,
    response_time: Arc<Mutex<TimeSeries>>,
    error_rate: Arc<Mutex<TimeSeries>>,
    cpu_usage: Arc<Mutex<TimeSeries>>,
    memory_usage: Arc<Mutex<TimeSeries>>,

    // Per-endpoint stats
    endpoint_stats: Arc<Mutex<HashMap<String, EndpointStats>>>,

    // System monitoring
    system: Arc<Mutex<System>>,

    // Counters
    total_requests: Arc<Mutex<u64>>,
    total_errors: Arc<Mutex<u64>>,
}

impl AdvancedMetrics {
    pub fn new() -> Self {
        let retention = Duration::from_secs(3600); // 1 hour
        let max_points = 3600; // 1 point per second for 1 hour

        Self {
            request_rate: Arc::new(Mutex::new(TimeSeries::new(retention, max_points))),
            response_time: Arc::new(Mutex::new(TimeSeries::new(retention, max_points))),
            error_rate: Arc::new(Mutex::new(TimeSeries::new(retention, max_points))),
            cpu_usage: Arc::new(Mutex::new(TimeSeries::new(retention, max_points))),
            memory_usage: Arc::new(Mutex::new(TimeSeries::new(retention, max_points))),
            endpoint_stats: Arc::new(Mutex::new(HashMap::new())),
            system: Arc::new(Mutex::new(System::new_all())),
            total_requests: Arc::new(Mutex::new(0)),
            total_errors: Arc::new(Mutex::new(0)),
        }
    }

    pub fn record_request(&self, path: String, duration: f64, is_error: bool) {
        // Update request count
        {
            let mut total = self.total_requests.lock().unwrap();
            *total += 1;
        }

        // Update error count
        if is_error {
            let mut total = self.total_errors.lock().unwrap();
            *total += 1;
        }

        // Update response time time-series
        {
            let mut series = self.response_time.lock().unwrap();
            series.add(duration);
        }

        // Update per-endpoint stats
        {
            let mut stats = self.endpoint_stats.lock().unwrap();

            // Check if we're at capacity before adding new endpoint
            if stats.len() >= MAX_ENDPOINTS && !stats.contains_key(&path) {
                // Log warning when at capacity
                eprintln!(
                    "[WARN] Endpoint stats at capacity ({}), evicting least accessed endpoint",
                    MAX_ENDPOINTS
                );

                // Evict least accessed endpoint (lowest request count)
                if let Some(least_accessed_path) = stats
                    .iter()
                    .min_by_key(|(_, endpoint_stat)| endpoint_stat.count)
                    .map(|(p, _)| p.clone())
                {
                    stats.remove(&least_accessed_path);
                }
            } else if stats.len() >= ENDPOINTS_WARNING_THRESHOLD && !stats.contains_key(&path) {
                // Log warning when approaching capacity
                eprintln!(
                    "[WARN] Endpoint stats approaching capacity: {}/{} ({}%)",
                    stats.len(),
                    MAX_ENDPOINTS,
                    (stats.len() * 100) / MAX_ENDPOINTS
                );
            }

            stats.entry(path.clone())
                .or_insert_with(|| EndpointStats::new(path))
                .add_request(duration);
        }
    }

    pub fn update_system_metrics(&self) {
        let mut system = self.system.lock().unwrap();
        system.refresh_cpu();
        system.refresh_memory();

        // CPU usage (average across all cores)
        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
        {
            let mut series = self.cpu_usage.lock().unwrap();
            series.add(cpu_usage);
        }

        // Memory usage (percentage)
        let used_memory = system.used_memory() as f64;
        let total_memory = system.total_memory() as f64;
        let memory_percent = if total_memory > 0.0 {
            (used_memory / total_memory) * 100.0
        } else {
            0.0
        };
        {
            let mut series = self.memory_usage.lock().unwrap();
            series.add(memory_percent);
        }
    }

    pub fn get_request_rate(&self, duration: Duration) -> f64 {
        let series = self.request_rate.lock().unwrap();
        let points = series.get_recent(duration);
        if points.is_empty() {
            return 0.0;
        }
        points.len() as f64 / duration.as_secs_f64()
    }

    pub fn get_avg_response_time(&self) -> f64 {
        let series = self.response_time.lock().unwrap();
        series.average()
    }

    pub fn get_response_time_percentile(&self, p: f64) -> f64 {
        let series = self.response_time.lock().unwrap();
        series.percentile(p)
    }

    pub fn get_error_rate(&self) -> f64 {
        let total_requests = *self.total_requests.lock().unwrap();
        let total_errors = *self.total_errors.lock().unwrap();

        if total_requests == 0 {
            return 0.0;
        }

        (total_errors as f64 / total_requests as f64) * 100.0
    }

    pub fn get_cpu_usage(&self) -> f64 {
        let series = self.cpu_usage.lock().unwrap();
        series.average()
    }

    pub fn get_memory_usage(&self) -> f64 {
        let series = self.memory_usage.lock().unwrap();
        series.average()
    }

    pub fn get_endpoint_stats(&self) -> Vec<EndpointStats> {
        let stats = self.endpoint_stats.lock().unwrap();
        let mut result: Vec<EndpointStats> = stats.values().cloned().collect();
        result.sort_by(|a, b| b.count.cmp(&a.count));
        result
    }

    pub fn get_cpu_trend(&self, duration: Duration) -> Vec<DataPoint> {
        let series = self.cpu_usage.lock().unwrap();
        series.get_recent(duration)
    }

    pub fn get_memory_trend(&self, duration: Duration) -> Vec<DataPoint> {
        let series = self.memory_usage.lock().unwrap();
        series.get_recent(duration)
    }

    pub fn get_response_time_trend(&self, duration: Duration) -> Vec<DataPoint> {
        let series = self.response_time.lock().unwrap();
        series.get_recent(duration)
    }
}

impl Clone for AdvancedMetrics {
    fn clone(&self) -> Self {
        Self {
            request_rate: Arc::clone(&self.request_rate),
            response_time: Arc::clone(&self.response_time),
            error_rate: Arc::clone(&self.error_rate),
            cpu_usage: Arc::clone(&self.cpu_usage),
            memory_usage: Arc::clone(&self.memory_usage),
            endpoint_stats: Arc::clone(&self.endpoint_stats),
            system: Arc::clone(&self.system),
            total_requests: Arc::clone(&self.total_requests),
            total_errors: Arc::clone(&self.total_errors),
        }
    }
}
