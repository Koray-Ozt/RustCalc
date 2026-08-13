use rust_calc::{HistoryStore, Operator};

fn temp_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rust-calc-{name}-{}", std::process::id()))
}

#[test]
fn calculation_history_survives_a_restart() {
    let path = temp_dir("history");
    let _ = std::fs::remove_dir_all(&path);

    {
        let mut history = HistoryStore::open(&path).unwrap();
        history.record(8.0, Operator::Divide, 2.0, 4.0).unwrap();
    }

    let history = HistoryStore::open(&path).unwrap();
    assert_eq!(history.entries(), vec!["8 ÷ 2 = 4"]);
    drop(history);
    std::fs::remove_dir_all(path).unwrap();
}
