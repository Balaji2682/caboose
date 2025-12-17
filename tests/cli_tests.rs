use caboose::cli::{Cli, Commands};
use clap::Parser;

#[test]
fn parses_dev_with_process() {
    let cli = Cli::parse_from(["caboose", "dev", "web"]);
    match cli.command {
        Some(Commands::Dev { process }) => assert_eq!(process, Some("web".into())),
        _ => panic!("Expected dev command"),
    }
}

#[test]
fn parses_logs_and_stop() {
    let cli = Cli::parse_from(["caboose", "logs", "worker"]);
    match cli.command {
        Some(Commands::Logs { process }) => assert_eq!(process, "worker"),
        _ => panic!("Expected logs command"),
    }

    let cli = Cli::parse_from(["caboose", "stop"]);
    assert!(matches!(cli.command, Some(Commands::Stop)));
}
