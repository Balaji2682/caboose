use caboose::git::GitInfo;

#[test]
fn formats_git_info_short() {
    let info = GitInfo {
        branch: Some("main".into()),
        has_changes: true,
        ahead: 2,
        behind: 1,
    };

    let formatted = info.format_short();
    assert!(formatted.contains("main"));
    assert!(formatted.contains("*"));
    assert!(formatted.contains("↑2"));
    assert!(formatted.contains("↓1"));
}
