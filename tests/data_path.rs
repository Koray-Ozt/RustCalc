use rust_calc::resolve_database_path;
use std::path::PathBuf;

#[test]
fn explicit_data_path_takes_precedence() {
    assert_eq!(
        resolve_database_path(Some("/tmp/custom".into()), Some("/tmp/xdg".into()), None),
        PathBuf::from("/tmp/custom")
    );
}

#[test]
fn platform_data_directory_is_used_by_default() {
    assert_eq!(
        resolve_database_path(None, Some("/tmp/xdg".into()), None),
        PathBuf::from("/tmp/xdg/rust-calc/history.ferrite")
    );
    assert_eq!(
        resolve_database_path(None, None, Some("/home/user".into())),
        PathBuf::from("/home/user/.local/share/rust-calc/history.ferrite")
    );
}

#[test]
fn invalid_xdg_data_directory_falls_back_to_home() {
    for invalid in [PathBuf::new(), PathBuf::from("relative/data")] {
        assert_eq!(
            resolve_database_path(None, Some(invalid), Some("/home/user".into())),
            PathBuf::from("/home/user/.local/share/rust-calc/history.ferrite")
        );
    }
}
