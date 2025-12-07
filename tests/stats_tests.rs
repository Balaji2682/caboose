use caboose::stats::{PerformanceStats, StatsCollector};

#[test]
fn performance_stats_calculations() {
    let mut stats = PerformanceStats::default();
    stats.total_requests = 2;
    stats.total_duration = 50.0;
    stats.error_count = 1;
    stats.sql_queries = 2;
    stats.total_sql_duration = 10.0;

    assert_eq!(stats.avg_response_time(), 25.0);
    assert_eq!(stats.error_rate(), 50.0);
    assert_eq!(stats.avg_sql_time(), 5.0);
}

#[test]
fn stats_collector_records_requests_and_sql() {
    let collector = StatsCollector::new();
    collector.record_request(200, 10.0);
    collector.record_request(500, 20.0);
    collector.record_sql_query(5.0);

    let stats = collector.get_stats();
    assert_eq!(stats.total_requests, 2);
    assert_eq!(stats.error_count, 1);
    assert_eq!(stats.status_codes.get(&500), Some(&1));
    assert_eq!(stats.sql_queries, 1);
    assert_eq!(stats.avg_response_time(), 15.0);
}
