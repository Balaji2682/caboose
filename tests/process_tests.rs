use caboose::process::ProcessManager;

#[test]
fn spawn_process_rejects_empty_command() {
    let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let manager = ProcessManager::new(_tx);
    let err = manager.spawn_process("web".into(), "".into(), std::collections::HashMap::new());
    assert!(err.is_err());
}
