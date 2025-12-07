use caboose::context::RequestContextTracker;
use caboose::parser::{HttpRequest, LogEvent, SqlQuery};

#[test]
fn tracker_collects_requests_and_queries() {
    let tracker = RequestContextTracker::new();

    tracker.process_log_event(&LogEvent::HttpRequest(HttpRequest {
        method: "GET".into(),
        path: "/users".into(),
        status: None,
        duration: None,
        controller: None,
        action: None,
    }));

    tracker.process_log_event(&LogEvent::SqlQuery(SqlQuery {
        query: r#"SELECT "users".* FROM "users" WHERE "users"."id" = 1"#.into(),
        duration: Some(5.0),
        rows: Some(1),
        name: Some("User Load".into()),
    }));

    tracker.process_log_event(&LogEvent::SqlQuery(SqlQuery {
        query: r#"SELECT "users".* FROM "users" WHERE "users"."id" = 1"#.into(),
        duration: Some(5.0),
        rows: Some(1),
        name: Some("User Load".into()),
    }));

    tracker.process_log_event(&LogEvent::SqlQuery(SqlQuery {
        query: r#"SELECT "users".* FROM "users" WHERE "users"."id" = 1"#.into(),
        duration: Some(5.0),
        rows: Some(1),
        name: Some("User Load".into()),
    }));

    tracker.process_log_event(&LogEvent::HttpRequest(HttpRequest {
        method: "GET".into(),
        path: "/users".into(),
        status: Some(200),
        duration: Some(30.0),
        controller: None,
        action: None,
    }));

    let completed = tracker.get_recent_requests();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].context.query_count(), 3);
    assert_eq!(completed[0].n_plus_one_issues.len(), 1);
}
