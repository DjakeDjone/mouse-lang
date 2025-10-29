use serde::{Deserialize, Serialize};

pub mod persistence;
pub mod query_engine;
pub mod row_schemaless;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DBValue {
    String(String),
    Number(f64),
    Timestamp(i64),
}

impl DBValue {
    pub fn vtype(&self) -> DBValueType {
        match self {
            DBValue::String(_) => DBValueType::String,
            DBValue::Number(_) => DBValueType::Number,
            DBValue::Timestamp(_) => DBValueType::Timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash, Deserialize)]
pub enum DBValueType {
    String,
    Number,
    Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterEntity {
    Equals(Box<FilterEntity>, Box<FilterEntity>),
    GreaterThan(Box<FilterEntity>, Box<FilterEntity>),
    LessThan(Box<FilterEntity>, Box<FilterEntity>),
    FuzzyMatch(Box<FilterEntity>, Box<FilterEntity>, u8), // Fuzzy match threshold

    Not(Box<FilterEntity>),
    And(Box<FilterEntity>, Box<FilterEntity>),
    Or(Box<FilterEntity>, Box<FilterEntity>),
    Xor(Box<FilterEntity>, Box<FilterEntity>),

    Value(DBValue),
    Column(String),
}

pub struct DBSettings {
    url: &'static str,
    cache_size: u32,
}

const DB_SETTINGS: DBSettings = DBSettings {
    url: "./mouse-src/data/database.db",
    cache_size: 100,
};
