use caboose::parser::{LogEvent, RailsLogParser};

#[test]
fn parses_http_start_and_completion() {
    let start = RailsLogParser::parse_line(r#"Started GET "/users/1" for 127.0.0.1"#);
    match start {
        Some(LogEvent::HttpRequest(req)) => {
            assert_eq!(req.method, "GET");
            assert_eq!(req.path, "/users/1");
            assert!(req.status.is_none());
        }
        _ => panic!("Expected HTTP start"),
    }

    let done = RailsLogParser::parse_line("Completed 200 OK in 45.7ms");
    match done {
        Some(LogEvent::HttpRequest(req)) => {
            assert_eq!(req.status, Some(200));
            assert_eq!(req.duration, Some(45.7));
        }
        _ => panic!("Expected HTTP completion"),
    }
}

#[test]
fn parses_sql_and_error_lines() {
    let sql = RailsLogParser::parse_line(r#"User Load (0.5ms)  SELECT "users".* FROM "users""#);
    match sql {
        Some(LogEvent::SqlQuery(q)) => {
            assert_eq!(q.name.as_deref(), Some("User Load"));
            assert_eq!(q.duration, Some(0.5));
        }
        _ => panic!("Expected SQL event"),
    }

    let error = RailsLogParser::parse_line("FATAL -- Exception in thread");
    assert!(matches!(error, Some(LogEvent::Error(_))));
}

#[test]
fn highlights_sql_keywords() {
    let highlighted = RailsLogParser::highlight_sql("SELECT * FROM users WHERE id = 1");
    assert!(highlighted.contains("[KW]SELECT[/KW]"));
    assert!(highlighted.contains("[KW]FROM[/KW]"));
}
