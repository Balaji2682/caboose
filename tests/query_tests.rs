use caboose::query::{
    NPlusOneDetector, PerformanceIssue, QueryAnalyzer, QueryFingerprint, QueryInfo, QueryType,
    RequestContext,
};

fn sample_select(duration: f64) -> QueryInfo {
    QueryInfo {
        raw_query: r#"SELECT "users".* FROM "users" WHERE "users"."id" = 1"#.to_string(),
        fingerprint: QueryFingerprint::new(
            r#"SELECT "users".* FROM "users" WHERE "users"."id" = 1"#,
        ),
        duration,
        rows: None,
        query_type: QueryType::Select,
    }
}

#[test]
fn fingerprint_normalizes_values() {
    let fp = QueryFingerprint::new("SELECT * FROM users WHERE id = 123 AND name = 'John'");
    assert_eq!(
        fp.normalized,
        "SELECT * FROM users WHERE id = ? AND name = ?"
    );
}

#[test]
fn query_type_detection() {
    assert_eq!(QueryType::from_sql("select *"), QueryType::Select);
    assert_eq!(QueryType::from_sql("UPDATE x"), QueryType::Update);
    assert_eq!(QueryType::from_sql("COMMIT"), QueryType::Commit);
    assert_eq!(QueryType::from_sql("ALTER TABLE"), QueryType::Other);
}

#[test]
fn n_plus_one_detector_flags_repeated_selects() {
    let mut ctx = RequestContext::new(Some("/users".into()));
    ctx.add_query(sample_select(2.0));
    ctx.add_query(sample_select(3.0));
    ctx.add_query(sample_select(4.0));

    let issues = NPlusOneDetector::detect(&ctx);
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert_eq!(issue.count, 3);
    assert!(issue.suggestion.contains("includes"));
}

#[test]
fn query_analyzer_flags_select_star_and_slow_queries() {
    let info = QueryInfo {
        raw_query: "SELECT * FROM users WHERE users.id = 1".to_string(),
        fingerprint: QueryFingerprint::new("SELECT * FROM users WHERE users.id = 1"),
        duration: 120.0,
        rows: Some(200),
        query_type: QueryType::Select,
    };

    let recs = QueryAnalyzer::analyze(&info);
    assert!(
        recs.iter()
            .any(|r| r.issue_type == PerformanceIssue::SelectStar)
    );
    assert!(
        recs.iter()
            .any(|r| r.issue_type == PerformanceIssue::SlowQuery)
    );
    assert!(
        recs.iter()
            .any(|r| r.issue_type == PerformanceIssue::LargeResultSet)
    );
    let slow = recs
        .iter()
        .find(|r| r.issue_type == PerformanceIssue::SlowQuery)
        .expect("missing slow query recommendation");
    assert!(slow.suggestion.contains("indexes"));
}
