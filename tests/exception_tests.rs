use caboose::exception::{ExceptionSeverity, ExceptionTracker};

#[test]
fn parses_exception_and_backtrace() {
    let tracker = ExceptionTracker::new();
    tracker.parse_line("NoMethodError: undefined method `foo' for nil:NilClass");
    tracker.parse_line("  app/models/user.rb:12:in `block in find'");
    tracker.parse_line("irrelevant line to end backtrace");

    let stats = tracker.get_stats();
    assert_eq!(stats.total_exceptions, 1);
    assert_eq!(stats.high_count, 1); // NoMethodError is High

    let groups = tracker.get_grouped_exceptions();
    assert_eq!(groups.len(), 1);
    assert_eq!(
        groups[0].sample_exception.file_path.as_deref(),
        Some("app/models/user.rb")
    );
    assert_eq!(groups[0].sample_exception.line_number, Some(12));
}

#[test]
fn groups_similar_exceptions() {
    let tracker = ExceptionTracker::new();
    tracker.parse_line("NameError: undefined local variable or method `user_123'");
    tracker.parse_line("  app/controllers/users_controller.rb:10:in `show'");
    tracker.parse_line("done");

    tracker.parse_line("NameError: undefined local variable or method `user_456'");
    tracker.parse_line("  app/controllers/users_controller.rb:11:in `show'");
    tracker.parse_line("done");

    let groups = tracker.get_grouped_exceptions();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].count, 2);
    assert_eq!(
        ExceptionSeverity::from_exception_type(&groups[0].exception_type),
        ExceptionSeverity::High
    );
    assert!(tracker.get_exception_rate() >= 2.0);
}
