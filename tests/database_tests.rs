use caboose::database::{DatabaseHealth, IssueType};

#[test]
fn tracks_slow_queries_and_tables() {
    let db = DatabaseHealth::new();
    let q = r#"SELECT * FROM "users" WHERE "users"."id" = 1"#;

    db.analyze_query(q, 120.0);
    db.analyze_query(q, 130.0);

    let slow = db.get_slow_queries();
    assert_eq!(slow.len(), 1);
    assert_eq!(slow[0].execution_count, 2);
    assert_eq!(slow[0].duration, 130.0);

    let stats = db.get_stats();
    assert_eq!(stats.tables_accessed.get("users"), Some(&2));
    assert_eq!(stats.select_star_count, 2);
}

#[test]
fn generates_issues_and_health_score() {
    let db = DatabaseHealth::new();
    // create 11 slow queries with WHERE to trigger slow + missing index issues
    for _ in 0..11 {
        db.analyze_query(r#"SELECT name FROM "users" WHERE "users"."id" = 1"#, 120.0);
    }

    let issues = db.get_issues();
    assert!(issues.iter().any(|i| i.issue_type == IssueType::SlowQuery));
    assert!(
        issues
            .iter()
            .any(|i| i.issue_type == IssueType::MissingIndex)
    );

    let score = db.calculate_health_score();
    assert!(score < 100);
}

#[test]
fn perfect_health_when_no_issues() {
    let db = DatabaseHealth::new();
    db.analyze_query("SELECT id FROM users", 10.0);
    db.analyze_query("SELECT name FROM users", 10.0);
    assert_eq!(db.calculate_health_score(), 100);
}
