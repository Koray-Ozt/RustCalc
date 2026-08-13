use rust_calc::{HistoryStore, Language, Operator};

fn temp_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rust-calc-{name}-{}", std::process::id()))
}

#[test]
fn calculation_history_survives_a_restart() {
    let path = temp_dir("history-restart");
    let _ = std::fs::remove_dir_all(&path);

    {
        let mut history = HistoryStore::open(&path).unwrap();
        history.record(8.0, Operator::Divide, 2.0, 4.0).unwrap();
    }

    let history = HistoryStore::open(&path).unwrap();
    assert_eq!(history.entry_texts(), vec!["8 ÷ 2 = 4"]);
    drop(history);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn new_databases_use_the_versioned_ferritedb_format() {
    let path = temp_dir("versioned-format");
    let _ = std::fs::remove_dir_all(&path);

    let history = HistoryStore::open(&path).unwrap();
    drop(history);

    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.join("format.json")).unwrap()).unwrap();
    assert_eq!(manifest, serde_json::json!({ "format": 1 }));

    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn history_clear_and_stats() {
    let path = temp_dir("history-stats");
    let _ = std::fs::remove_dir_all(&path);

    let mut history = HistoryStore::open(&path).unwrap();
    history.record(10.0, Operator::Add, 5.0, 15.0).unwrap();
    history.record(20.0, Operator::Multiply, 2.0, 40.0).unwrap();
    history.record(12.0, Operator::Add, 3.0, 15.0).unwrap();

    let stats = history.stats();
    assert_eq!(stats.total_count, 3);
    assert_eq!(stats.add_count, 2);
    assert_eq!(stats.multiply_count, 1);
    assert_eq!(stats.most_used_operator(), Some(Operator::Add));

    history.clear().unwrap();
    assert_eq!(history.entries().len(), 0);
    assert_eq!(history.entry_texts().len(), 0);
    drop(history);

    let reloaded = HistoryStore::open(&path).unwrap();
    assert_eq!(reloaded.entries().len(), 0);

    drop(reloaded);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn language_preference_persists_across_restart() {
    let path = temp_dir("lang-persist");
    let _ = std::fs::remove_dir_all(&path);

    {
        let mut history = HistoryStore::open(&path).unwrap();
        assert_eq!(history.language(), Language::Turkish);
        history.set_language(Language::Russian).unwrap();
    }

    let reloaded = HistoryStore::open(&path).unwrap();
    assert_eq!(reloaded.language(), Language::Russian);

    drop(reloaded);
    std::fs::remove_dir_all(path).unwrap();
}
