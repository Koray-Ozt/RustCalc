use ferrite_core::Database;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Turkish,
    English,
    Russian,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::Turkish => "tr",
            Self::English => "en",
            Self::Russian => "ru",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Turkish => "TR - Türkçe",
            Self::English => "EN - English",
            Self::Russian => "RU - Русский",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "en" => Self::English,
            "ru" => Self::Russian,
            _ => Self::Turkish,
        }
    }

    pub fn all() -> &'static [Language] {
        &[Language::Turkish, Language::English, Language::Russian]
    }
}

pub struct I18n;

impl I18n {
    pub fn history_empty(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "FerriteDB geçmişi boş",
            Language::English => "FerriteDB history is empty",
            Language::Russian => "История FerriteDB пуста",
        }
    }

    pub fn no_history_records(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Henüz kaydedilmiş bir işlem bulunmuyor.",
            Language::English => "No recorded transactions found.",
            Language::Russian => "Записи истории пока отсутствуют.",
        }
    }

    pub fn division_by_zero(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Sıfıra bölünemez",
            Language::English => "Cannot divide by zero",
            Language::Russian => "Деление на ноль невозможно",
        }
    }

    pub fn invalid_number(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Geçersiz sayı",
            Language::English => "Invalid number",
            Language::Russian => "Недействительное число",
        }
    }

    pub fn result_too_large(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Sonuç çok büyük",
            Language::English => "Result is too large",
            Language::Russian => "Результат слишком велик",
        }
    }

    pub fn history_btn_label(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "📜 Geçmiş & İstatistikler (Ctrl+H)",
            Language::English => "📜 History & Stats (Ctrl+H)",
            Language::Russian => "📜 История и статистика (Ctrl+H)",
        }
    }

    pub fn history_dialog_title(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "FerriteDB Geçmişi & İstatistikler",
            Language::English => "FerriteDB History & Statistics",
            Language::Russian => "История и статистика FerriteDB",
        }
    }

    pub fn analytics_title(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "📊 FerriteDB Kullanım Analitiği",
            Language::English => "📊 FerriteDB Usage Analytics",
            Language::Russian => "📊 Аналитика использования FerriteDB",
        }
    }

    pub fn total_count(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Toplam Kayıtlı İşlem",
            Language::English => "Total Recorded Operations",
            Language::Russian => "Всего записей операций",
        }
    }

    pub fn favorite_operator(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Favori Operatör",
            Language::English => "Favorite Operator",
            Language::Russian => "Любимый оператор",
        }
    }

    pub fn none(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Yok",
            Language::English => "None",
            Language::Russian => "Нет",
        }
    }

    pub fn breakdown(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Dağılım",
            Language::English => "Breakdown",
            Language::Russian => "Распределение",
        }
    }

    pub fn history_list_title(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "İşlem Geçmişi (Son Kayıttan İlke):",
            Language::English => "Transaction History (Latest First):",
            Language::Russian => "История операций (сначала новые):",
        }
    }

    pub fn clear_history(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "🗑️ Geçmişi Sıfırla",
            Language::English => "🗑️ Clear History",
            Language::Russian => "🗑️ Очистить историю",
        }
    }

    pub fn close(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Kapat",
            Language::English => "Close",
            Language::Russian => "Закрыть",
        }
    }

    pub fn app_title(lang: Language) -> &'static str {
        match lang {
            Language::Turkish => "Ferrite Hesap Makinesi",
            Language::English => "Ferrite Calculator",
            Language::Russian => "Калькулятор Ferrite",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub left: f64,
    pub operator: Operator,
    pub right: f64,
    pub result: f64,
    pub timestamp: u64,
}

impl HistoryEntry {
    pub fn formatted(&self) -> String {
        format!(
            "{} {} {} = {}",
            format_number(self.left),
            self.operator.symbol(),
            format_number(self.right),
            format_number(self.result)
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryStats {
    pub total_count: usize,
    pub add_count: usize,
    pub subtract_count: usize,
    pub multiply_count: usize,
    pub divide_count: usize,
}

impl HistoryStats {
    pub fn most_used_operator(&self) -> Option<Operator> {
        let ops = [
            (Operator::Add, self.add_count),
            (Operator::Subtract, self.subtract_count),
            (Operator::Multiply, self.multiply_count),
            (Operator::Divide, self.divide_count),
        ];

        ops.into_iter()
            .filter(|&(_, count)| count > 0)
            .max_by_key(|&(_, count)| count)
            .map(|(op, _)| op)
    }
}

pub struct HistoryStore {
    db: Database,
    entries: Vec<HistoryEntry>,
    language: Language,
}

impl HistoryStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ferrite_core::Error> {
        let db = Database::open(path)?;
        let mut entries = Vec::new();
        let mut language = Language::Turkish;

        if let Ok(settings) = db.list(Some("settings/")) {
            for (key, val) in settings {
                if key == "settings/language" {
                    language = val.as_str().map(Language::from_code).unwrap_or(language);
                }
            }
        }

        let raw_list = db.list(Some("history/"))?;
        for (key, value) in raw_list {
            if let Ok(entry) = serde_json::from_value::<HistoryEntry>(value.clone()) {
                entries.push(entry);
            } else if let Some(text) = value.get("text").and_then(|v| v.as_str()) {
                entries.push(HistoryEntry {
                    id: key,
                    left: 0.0,
                    operator: Operator::Add,
                    right: 0.0,
                    result: 0.0,
                    timestamp: 0,
                });
                let _ = text;
            }
        }

        Ok(Self {
            db,
            entries,
            language,
        })
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn set_language(&mut self, lang: Language) -> Result<(), ferrite_core::Error> {
        self.db.put_key(
            "settings/language",
            serde_json::to_value(lang.code()).map_err(ferrite_core::Error::Json)?,
        )?;
        self.language = lang;
        Ok(())
    }

    pub fn record(
        &mut self,
        left: f64,
        operator: Operator,
        right: f64,
        result: f64,
    ) -> Result<HistoryEntry, ferrite_core::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let id = format!("history/{:020}_{}", timestamp, self.entries.len());
        let entry = HistoryEntry {
            id: id.clone(),
            left,
            operator,
            right,
            result,
            timestamp,
        };

        self.db.put_key(
            &id,
            serde_json::to_value(&entry).map_err(ferrite_core::Error::Json)?,
        )?;
        self.entries.push(entry.clone());
        Ok(entry)
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn entry_texts(&self) -> Vec<String> {
        self.entries.iter().map(HistoryEntry::formatted).collect()
    }

    pub fn clear(&mut self) -> Result<(), ferrite_core::Error> {
        let keys_to_delete: Vec<String> = self
            .db
            .list(Some("history/"))?
            .into_iter()
            .map(|(k, _)| k)
            .collect();

        for key in keys_to_delete {
            self.db.delete_key(&key)?;
        }

        self.entries.clear();
        Ok(())
    }

    pub fn stats(&self) -> HistoryStats {
        let mut stats = HistoryStats {
            total_count: self.entries.len(),
            ..Default::default()
        };

        for entry in &self.entries {
            match entry.operator {
                Operator::Add => stats.add_count += 1,
                Operator::Subtract => stats.subtract_count += 1,
                Operator::Multiply => stats.multiply_count += 1,
                Operator::Divide => stats.divide_count += 1,
            }
        }

        stats
    }
}

pub fn calculate(
    left: f64,
    operator: Operator,
    right: f64,
    lang: Language,
) -> Result<f64, &'static str> {
    match operator {
        Operator::Add => Ok(left + right),
        Operator::Subtract => Ok(left - right),
        Operator::Multiply => Ok(left * right),
        Operator::Divide if right == 0.0 => Err(I18n::division_by_zero(lang)),
        Operator::Divide => Ok(left / right),
    }
}
