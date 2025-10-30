use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub mod query_engine;
pub mod row_schemaless;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub enum DBValue {
    String(String),
    Number(f64),
    Timestamp(i64),
    Null,
}

impl DBValue {
    pub fn vtype(&self) -> DBValueType {
        match self {
            DBValue::String(_) => DBValueType::String,
            DBValue::Number(_) => DBValueType::Number,
            DBValue::Timestamp(_) => DBValueType::Timestamp,
            DBValue::Null => DBValueType::Null,
        }
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, Encode, Decode)]
pub enum DBValueType {
    String,
    Number,
    Timestamp,
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
    Null,
}
