use std::fs;
use std::path::PathBuf;

use caboose::rails::RailsApp;

fn temp_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let uniq = format!(
        "caboose_rails_{}_{}",
        name,
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    dir.push(uniq);
    dir
}

#[test]
fn detects_rails_app_and_features() {
    let root = temp_dir("app");
    fs::create_dir_all(root.join("config")).unwrap();
    fs::write(
        root.join("Gemfile"),
        "gem 'rails'\ngem 'sidekiq'\ngem 'vite_rails'",
    )
    .unwrap();
    fs::write(root.join("config/application.rb"), "module App end").unwrap();
    fs::write(root.join("config/database.yml"), "adapter: postgresql").unwrap();

    let app = RailsApp::detect_in_path(&root);
    assert!(app.detected);
    assert_eq!(app.database.as_deref(), Some("postgresql"));
    assert_eq!(app.background_job.as_deref(), Some("sidekiq"));
    assert_eq!(app.asset_pipeline.as_deref(), Some("vite"));
    assert!(app.generate_procfile(None).contains("bundle exec sidekiq"));

    let _ = fs::remove_dir_all(root);
}
