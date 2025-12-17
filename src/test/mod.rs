use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum TestFramework {
    RSpec,
    Minitest,
    TestUnit,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub status: TestStatus,
    pub duration: Option<f64>,
    pub failure_message: Option<String>,
    pub backtrace: Option<Vec<String>>,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Pending,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct TestRun {
    pub framework: TestFramework,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub pending: usize,
    pub skipped: usize,
    pub duration: Option<f64>,
    pub test_results: Vec<TestResult>,
}

impl TestRun {
    pub fn new(framework: TestFramework) -> Self {
        Self {
            framework,
            started_at: Instant::now(),
            completed_at: None,
            total_tests: 0,
            passed: 0,
            failed: 0,
            pending: 0,
            skipped: 0,
            duration: None,
            test_results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: TestResult) {
        self.total_tests += 1;
        match result.status {
            TestStatus::Passed => self.passed += 1,
            TestStatus::Failed => self.failed += 1,
            TestStatus::Pending => self.pending += 1,
            TestStatus::Skipped => self.skipped += 1,
        }
        self.test_results.push(result);
    }

    pub fn complete(&mut self, duration: Option<f64>) {
        self.completed_at = Some(Instant::now());
        self.duration = duration;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total_tests as f64) * 100.0
    }

    pub fn failed_tests(&self) -> Vec<&TestResult> {
        self.test_results
            .iter()
            .filter(|t| t.status == TestStatus::Failed)
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TestStats {
    pub total_runs: usize,
    pub total_tests_run: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_pending: usize,
    pub average_duration: Option<f64>,
    pub slowest_tests: Vec<TestResult>,
}

pub struct TestTracker {
    framework: Arc<Mutex<Option<TestFramework>>>,
    current_run: Arc<Mutex<Option<TestRun>>>,
    recent_runs: Arc<Mutex<Vec<TestRun>>>,
    stats: Arc<Mutex<TestStats>>,
    debugger_active: Arc<Mutex<bool>>,
    debugger_info: Arc<Mutex<Option<DebuggerInfo>>>,
}

#[derive(Debug, Clone)]
pub struct DebuggerInfo {
    pub debugger_type: DebuggerType,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub variables: HashMap<String, String>,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DebuggerType {
    Pry,
    Byebug,
    Debug,
    Unknown,
}

impl TestTracker {
    pub fn new() -> Self {
        Self {
            framework: Arc::new(Mutex::new(None)),
            current_run: Arc::new(Mutex::new(None)),
            recent_runs: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(TestStats::default())),
            debugger_active: Arc::new(Mutex::new(false)),
            debugger_info: Arc::new(Mutex::new(None)),
        }
    }

    pub fn detect_framework(&self, line: &str) -> Option<TestFramework> {
        if line.contains("RSpec") || line.contains("rspec") {
            Some(TestFramework::RSpec)
        } else if line.contains("Minitest") || line.contains("minitest") {
            Some(TestFramework::Minitest)
        } else if line.contains("Test::Unit") {
            Some(TestFramework::TestUnit)
        } else {
            None
        }
    }

    pub fn start_test_run(&self, framework: TestFramework) {
        let mut current = self.current_run.lock().unwrap();
        *current = Some(TestRun::new(framework.clone()));

        let mut fw = self.framework.lock().unwrap();
        *fw = Some(framework);
    }

    pub fn add_test_result(&self, result: TestResult) {
        let mut current = self.current_run.lock().unwrap();
        if let Some(ref mut run) = *current {
            run.add_result(result);
        }
    }

    pub fn complete_test_run(&self, duration: Option<f64>) {
        let mut current = self.current_run.lock().unwrap();
        if let Some(ref mut run) = *current {
            run.complete(duration);

            // Update stats
            let mut stats = self.stats.lock().unwrap();
            stats.total_runs += 1;
            stats.total_tests_run += run.total_tests;
            stats.total_passed += run.passed;
            stats.total_failed += run.failed;
            stats.total_pending += run.pending;

            // Update average duration
            if let Some(dur) = duration {
                stats.average_duration = Some(
                    stats
                        .average_duration
                        .map(|avg| (avg + dur) / 2.0)
                        .unwrap_or(dur),
                );
            }

            // Update slowest tests
            for test in &run.test_results {
                if let Some(test_dur) = test.duration {
                    if test_dur > 100.0 {
                        stats.slowest_tests.push(test.clone());
                    }
                }
            }
            stats
                .slowest_tests
                .sort_by(|a, b| b.duration.partial_cmp(&a.duration).unwrap());
            stats.slowest_tests.truncate(20);

            // Store in recent runs
            let mut recent = self.recent_runs.lock().unwrap();
            recent.push(run.clone());
            if recent.len() > 10 {
                recent.remove(0);
            }
        }

        *current = None;
    }

    pub fn parse_line(&self, line: &str) {
        // Auto-detect framework if not set
        if self.framework.lock().unwrap().is_none() {
            if let Some(fw) = self.detect_framework(line) {
                self.start_test_run(fw);
            }
        }

        // Check for debugger activation
        self.detect_debugger(line);

        // Parse test output based on framework
        let framework = self.framework.lock().unwrap().clone();
        match framework {
            Some(TestFramework::RSpec) => self.parse_rspec_line(line),
            Some(TestFramework::Minitest) => self.parse_minitest_line(line),
            _ => {}
        }
    }

    fn parse_rspec_line(&self, line: &str) {
        // RSpec example format: "  example description"
        // Failure format: "  1) example description"
        // Summary format: "1 example, 0 failures"

        // Check for test completion summary
        if line.contains("example") && (line.contains("failure") || line.contains("passed")) {
            if let Some(duration) = Self::extract_duration_rspec(line) {
                self.complete_test_run(Some(duration));
            }
        }

        // Check for failures
        if line.trim().starts_with("1)") || line.trim().starts_with("2)") {
            // This is a failure description line
            // Next lines will contain details
        }
    }

    fn parse_minitest_line(&self, line: &str) {
        // Minitest format: "Finished in 0.123s"
        // Results: "1 runs, 2 assertions, 0 failures, 0 errors, 0 skips"

        if line.contains("Finished in") {
            if let Some(duration) = Self::extract_duration_minitest(line) {
                self.complete_test_run(Some(duration * 1000.0)); // Convert to ms
            }
        }

        // Parse results line
        if line.contains("runs,") && line.contains("assertions,") {
            self.parse_minitest_results(line);
        }
    }

    fn parse_minitest_results(&self, line: &str) {
        // Example: "1 runs, 2 assertions, 0 failures, 0 errors, 0 skips"
        let parts: Vec<&str> = line.split(',').collect();

        for part in parts {
            let part = part.trim();
            if part.contains("failure") {
                if let Some(count) = part
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse::<usize>().ok())
                {
                    for _ in 0..count {
                        self.add_test_result(TestResult {
                            test_name: "Unknown test".to_string(),
                            file_path: None,
                            line_number: None,
                            status: TestStatus::Failed,
                            duration: None,
                            failure_message: None,
                            backtrace: None,
                            timestamp: Instant::now(),
                        });
                    }
                }
            }
        }
    }

    fn extract_duration_rspec(line: &str) -> Option<f64> {
        // Format: "Finished in 0.12345 seconds"
        if let Some(pos) = line.find("Finished in") {
            let after = &line[pos + 12..];
            if let Some(end) = after.find("second") {
                let duration_str = &after[..end].trim();
                return duration_str.parse::<f64>().ok().map(|s| s * 1000.0);
            }
        }
        None
    }

    fn extract_duration_minitest(line: &str) -> Option<f64> {
        // Format: "Finished in 0.123456s"
        if let Some(pos) = line.find("Finished in") {
            let after = &line[pos + 12..];
            if let Some(end) = after.find('s') {
                let duration_str = &after[..end].trim();
                return duration_str.parse::<f64>().ok();
            }
        }
        None
    }

    fn detect_debugger(&self, line: &str) {
        let mut active = self.debugger_active.lock().unwrap();
        let mut info = self.debugger_info.lock().unwrap();

        // Detect Pry
        if line.contains("pry(") || line.contains("Frame number:") {
            *active = true;
            *info = Some(DebuggerInfo {
                debugger_type: DebuggerType::Pry,
                file_path: Self::extract_file_path(line),
                line_number: Self::extract_line_number(line),
                variables: HashMap::new(),
                timestamp: Instant::now(),
            });
        }
        // Detect Byebug
        else if line.contains("byebug") || line.contains("[byebug]") {
            *active = true;
            *info = Some(DebuggerInfo {
                debugger_type: DebuggerType::Byebug,
                file_path: Self::extract_file_path(line),
                line_number: Self::extract_line_number(line),
                variables: HashMap::new(),
                timestamp: Instant::now(),
            });
        }
        // Detect debug gem
        else if line.contains("DEBUGGER:") || line.contains("debug.rb") {
            *active = true;
            *info = Some(DebuggerInfo {
                debugger_type: DebuggerType::Debug,
                file_path: Self::extract_file_path(line),
                line_number: Self::extract_line_number(line),
                variables: HashMap::new(),
                timestamp: Instant::now(),
            });
        }
    }

    fn extract_file_path(line: &str) -> Option<String> {
        // Try to extract file path from various formats
        // Format: "From: /path/to/file.rb:123"
        if let Some(pos) = line.find("From:") {
            let after = &line[pos + 5..].trim();
            if let Some(colon) = after.find(':') {
                return Some(after[..colon].to_string());
            }
        }
        None
    }

    fn extract_line_number(line: &str) -> Option<usize> {
        // Try to extract line number
        // Format: "/path/to/file.rb:123"
        if let Some(pos) = line.rfind(':') {
            let after = &line[pos + 1..];
            if let Some(num_str) = after.split_whitespace().next() {
                return num_str.parse::<usize>().ok();
            }
        }
        None
    }

    pub fn get_current_run(&self) -> Option<TestRun> {
        self.current_run.lock().unwrap().clone()
    }

    pub fn get_recent_runs(&self) -> Vec<TestRun> {
        self.recent_runs.lock().unwrap().clone()
    }

    pub fn get_stats(&self) -> TestStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn is_debugger_active(&self) -> bool {
        *self.debugger_active.lock().unwrap()
    }

    pub fn get_debugger_info(&self) -> Option<DebuggerInfo> {
        self.debugger_info.lock().unwrap().clone()
    }

    pub fn clear_debugger(&self) {
        *self.debugger_active.lock().unwrap() = false;
        *self.debugger_info.lock().unwrap() = None;
    }
}
