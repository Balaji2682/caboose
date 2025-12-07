use caboose::explain::{ExplainExecutor, WarningSeverity};

#[test]
fn explain_executor_simulates_plan_with_warnings() {
    let exec = ExplainExecutor::new(None);
    let plan = exec.explain_query("SELECT * FROM users").unwrap();

    assert!(plan.has_seq_scan());
    assert!(!plan.has_index_scan());
    assert!(plan.suggest_indexes().iter().any(|s| s.contains("index")));

    let severities: Vec<_> = plan.warnings.iter().map(|w| w.severity.clone()).collect();
    assert!(severities.contains(&WarningSeverity::Warning));
}
