use ferrite_core::Database;
use serde_json::json;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operator {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "−",
            Self::Multiply => "×",
            Self::Divide => "÷",
        }
    }
}

pub fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".into();
    }

    value.to_string().replace('.', ",")
}

pub struct HistoryStore {
    db: Database,
    entries: Vec<String>,
}

impl HistoryStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ferrite_core::Error> {
        let db = Database::open(path)?;
        let entries = db
            .list(Some("history/"))?
            .into_iter()
            .filter_map(|(_, value)| value.get("text")?.as_str().map(str::to_owned))
            .collect();
        Ok(Self { db, entries })
    }

    pub fn record(
        &mut self,
        left: f64,
        operator: Operator,
        right: f64,
        result: f64,
    ) -> Result<(), ferrite_core::Error> {
        let text = format!(
            "{} {} {} = {}",
            format_number(left),
            operator.symbol(),
            format_number(right),
            format_number(result)
        );
        let key = format!("history/{:020}", self.entries.len());
        self.db.put_key(&key, json!({ "text": text }))?;
        self.entries.push(text);
        Ok(())
    }

    pub fn entries(&self) -> Vec<String> {
        self.entries.clone()
    }
}

pub fn calculate(left: f64, operator: Operator, right: f64) -> Result<f64, &'static str> {
    match operator {
        Operator::Add => Ok(left + right),
        Operator::Subtract => Ok(left - right),
        Operator::Multiply => Ok(left * right),
        Operator::Divide if right == 0.0 => Err("Sıfıra bölünemez"),
        Operator::Divide => Ok(left / right),
    }
}
