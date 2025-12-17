use std::fs;
use std::path::PathBuf;

use caboose::frontend::{FrontendApp, FrontendFramework, PackageManager};

fn temp_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let uniq = format!(
        "caboose_frontend_{}_{}",
        name,
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    dir.push(uniq);
    dir
}

#[test]
fn detects_vite_frontend_and_package_manager() {
    let root = temp_dir("vite");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("package.json"), r#"{"name":"demo"}"#).unwrap();
    fs::write(root.join("vite.config.js"), "export default {}").unwrap();
    fs::write(root.join("yarn.lock"), "").unwrap();

    let app = FrontendApp::detect_with_config(Some(root.to_str().unwrap()));
    assert!(app.detected);
    assert_eq!(app.framework, Some(FrontendFramework::Vite));
    assert_eq!(app.package_manager, PackageManager::Yarn);
    assert!(
        app.generate_procfile_entry(None)
            .unwrap()
            .contains("yarn run dev")
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn package_manager_detection_defaults_to_npm() {
    let root = temp_dir("npm");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("package.json"), "{}").unwrap();
    fs::write(root.join("next.config.js"), "").unwrap();

    let app = FrontendApp::detect_with_config(Some(root.to_str().unwrap()));
    assert_eq!(app.package_manager, PackageManager::Npm);
    assert_eq!(app.framework.unwrap().default_port(), 3000);

    let _ = fs::remove_dir_all(root);
}
