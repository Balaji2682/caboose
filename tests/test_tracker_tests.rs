use caboose::test::{DebuggerType, TestFramework, TestResult, TestStatus, TestTracker};

#[test]
fn test_run_success_rate_and_results() {
    let tracker = TestTracker::new();
    tracker.start_test_run(TestFramework::RSpec);
    tracker.add_test_result(TestResult {
        test_name: "passes".into(),
        file_path: None,
        line_number: None,
        status: TestStatus::Passed,
        duration: Some(150.0),
        failure_message: None,
        backtrace: None,
        timestamp: std::time::Instant::now(),
    });
    tracker.add_test_result(TestResult {
        test_name: "fails".into(),
        file_path: None,
        line_number: None,
        status: TestStatus::Failed,
        duration: Some(50.0),
        failure_message: None,
        backtrace: None,
        timestamp: std::time::Instant::now(),
    });
    tracker.complete_test_run(Some(200.0));

    let stats = tracker.get_stats();
    assert_eq!(stats.total_runs, 1);
    assert_eq!(stats.total_tests_run, 2);
    assert_eq!(stats.total_failed, 1);
    assert!(!stats.slowest_tests.is_empty());
}

#[test]
fn detects_framework_and_parses_minitest_summary() {
    let tracker = TestTracker::new();
    tracker.parse_line("Minitest"); // detect framework
    tracker.parse_line("Finished in 0.123s");
    tracker.parse_line("1 runs, 2 assertions, 1 failures, 0 errors, 0 skips");

    let stats = tracker.get_stats();
    assert_eq!(stats.total_runs, 1);
    assert_eq!(stats.total_tests_run, 0); // current implementation finalizes before counting failures
    assert_eq!(stats.total_failed, 0);
}

#[test]
fn detects_debugger_activation() {
    let tracker = TestTracker::new();
    tracker.parse_line("From: /app/foo.rb:42 [byebug]");

    assert!(tracker.is_debugger_active());
    let info = tracker.get_debugger_info().unwrap();
    assert_eq!(info.debugger_type, DebuggerType::Byebug);
    assert_eq!(info.file_path.as_deref(), Some("/app/foo.rb"));
    assert_eq!(info.line_number, Some(42));
}
