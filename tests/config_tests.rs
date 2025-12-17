use std::fs;
use std::path::PathBuf;

use caboose::config::{CabooseConfig, Procfile, load_env};

fn temp_path(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let unique = format!(
        "caboose_cfg_{}_{}",
        name,
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    dir.push(unique);
    dir
}

#[test]
fn parse_procfile_content_and_errors() {
    let ok = Procfile::parse_content("web: bundle exec rails s\nworker: sidekiq").unwrap();
    assert_eq!(ok.processes.len(), 2);
    assert_eq!(ok.processes[0].name, "web");
    assert_eq!(ok.processes[1].command, "sidekiq");

    let err = Procfile::parse_content("# only comments\n   \n");
    assert!(err.is_err());
}

#[test]
fn load_env_parses_values() {
    let path = temp_path("env");
    fs::write(&path, "FOO=bar\n#comment\nNUMBER=\"123\"\nINVALID\n").unwrap();

    let env = load_env(&path).unwrap();
    assert_eq!(env.get("FOO"), Some(&"bar".to_string()));
    assert_eq!(env.get("NUMBER"), Some(&"123".to_string()));
    assert!(env.get("INVALID").is_none());

    let _ = fs::remove_file(path);
}

#[test]
fn caboose_config_defaults_when_missing() {
    let cfg = CabooseConfig::load();
    // No file -> defaults for nested structs and overrides map
    assert!(cfg.frontend.path.is_none());
    assert!(!cfg.frontend.disable_auto_detect);
    assert_eq!(cfg.processes.len(), 0);
}

#[test]
fn caboose_config_create_example_has_sections() {
    let example = CabooseConfig::create_example();
    assert!(example.contains("[frontend]"));
    assert!(example.contains("[rails]"));
    assert!(example.contains("process_name"));
}
