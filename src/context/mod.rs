use crate::parser::{HttpRequest, LogEvent, SqlQuery};
use crate::query::{
    NPlusOneDetector, NPlusOneIssue, QueryFingerprint, QueryInfo, QueryType, RequestContext,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Tracks request contexts and groups queries by request
pub struct RequestContextTracker {
    current_requests: Arc<Mutex<HashMap<String, RequestContext>>>,
    completed_requests: Arc<Mutex<Vec<CompletedRequest>>>,
    max_completed: usize,
}

#[derive(Debug, Clone)]
pub struct CompletedRequest {
    pub context: RequestContext,
    pub n_plus_one_issues: Vec<NPlusOneIssue>,
    pub total_duration: Option<f64>,
    pub status: Option<u16>,
    pub completed_at: Instant,
}

impl RequestContextTracker {
    pub fn new() -> Self {
        Self {
            current_requests: Arc::new(Mutex::new(HashMap::new())),
            completed_requests: Arc::new(Mutex::new(Vec::new())),
            max_completed: 100,
        }
    }

    pub fn process_log_event(&self, event: &LogEvent) {
        match event {
            LogEvent::HttpRequest(req) => {
                if req.status.is_none() {
                    // Request started
                    self.start_request(req);
                } else {
                    // Request completed
                    self.complete_request(req);
                }
            }
            LogEvent::SqlQuery(query) => {
                self.add_query_to_current_request(query);
            }
            _ => {}
        }
    }

    fn start_request(&self, req: &HttpRequest) {
        let path = req.path.clone();
        if path.is_empty() {
            return;
        }

        let context = RequestContext::new(Some(path.clone()));
        let mut requests = self.current_requests.lock().unwrap();
        requests.insert(path, context);
    }

    fn add_query_to_current_request(&self, sql_query: &SqlQuery) {
        let mut requests = self.current_requests.lock().unwrap();

        // If we have an active request, add the query to it
        // Otherwise, add it to a default "background" context
        if let Some((_path, context)) = requests.iter_mut().next() {
            let query_info = QueryInfo {
                raw_query: sql_query.query.clone(),
                fingerprint: QueryFingerprint::new(&sql_query.query),
                duration: sql_query.duration.unwrap_or(0.0),
                rows: sql_query.rows,
                query_type: QueryType::from_sql(&sql_query.query),
            };

            context.add_query(query_info);
        }
    }

    fn complete_request(&self, req: &HttpRequest) {
        let mut requests = self.current_requests.lock().unwrap();

        // Find the matching request context
        // Since we don't have exact path matching, take the first one
        if let Some((_path, context)) = requests.drain().next() {
            // Detect N+1 issues
            let n_plus_one_issues = NPlusOneDetector::detect(&context);

            let completed = CompletedRequest {
                context,
                n_plus_one_issues,
                total_duration: req.duration,
                status: req.status,
                completed_at: Instant::now(),
            };

            let mut completed_requests = self.completed_requests.lock().unwrap();
            completed_requests.push(completed);

            // Keep only the most recent requests
            if completed_requests.len() > self.max_completed {
                completed_requests.remove(0);
            }
        }
    }

    pub fn get_recent_requests(&self) -> Vec<CompletedRequest> {
        let completed = self.completed_requests.lock().unwrap();
        completed.clone()
    }

    pub fn get_current_requests(&self) -> Vec<RequestContext> {
        let current = self.current_requests.lock().unwrap();
        current.values().cloned().collect()
    }

    pub fn get_all_n_plus_one_issues(&self) -> Vec<NPlusOneIssue> {
        let completed = self.completed_requests.lock().unwrap();
        completed
            .iter()
            .flat_map(|req| req.n_plus_one_issues.clone())
            .collect()
    }
}
