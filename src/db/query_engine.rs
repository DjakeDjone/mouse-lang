use std::collections::HashMap;

use crate::db::{DBValue, DBValueType, FilterEntity};

pub struct PreSelectedField {
    pub name: String,
    pub kind: DBValueType,
    pub range: Option<(u64, u64)>,
}

impl PreSelectedField {
    pub fn from_column(col: (String, DBValueType)) -> Self {
        Self {
            name: col.0,
            kind: col.1,
            range: None,
        }
    }
}

pub fn pre_select(query: &FilterEntity) -> Option<Vec<PreSelectedField>> {
    None
}

/// Execute a query on a set of fields
/// returns true if the query matches the fields
pub fn execute_query(query: &FilterEntity, field: &HashMap<String, DBValue>) -> bool {
    evaluate_filter(&query, &field)
}

/// Recursively evaluate a filter entity against the provided fields
fn evaluate_filter(filter: &FilterEntity, fields: &HashMap<String, DBValue>) -> bool {
    match filter {
        FilterEntity::Equals(left, right) => {
            match (
                evaluate_to_value(left, fields),
                evaluate_to_value(right, fields),
            ) {
                (Some(l), Some(r)) => values_equal(&l, &r),
                _ => false,
            }
        }
        FilterEntity::Not(inner) => !evaluate_filter(inner, fields),
        FilterEntity::And(left, right) => {
            evaluate_filter(left, fields) && evaluate_filter(right, fields)
        }
        FilterEntity::Or(left, right) => {
            evaluate_filter(left, fields) || evaluate_filter(right, fields)
        }
        FilterEntity::Xor(left, right) => {
            evaluate_filter(left, fields) ^ evaluate_filter(right, fields)
        }
        FilterEntity::Value(_) => false, // A standalone value doesn't make sense as a boolean filter
        FilterEntity::Column(_) => false, // A standalone column reference doesn't make sense as a boolean filter
    }
}

/// Evaluate a filter entity to a concrete value
fn evaluate_to_value(filter: &FilterEntity, fields: &HashMap<String, DBValue>) -> Option<DBValue> {
    match filter {
        FilterEntity::Value(val) => Some(val.clone()),
        FilterEntity::Column(name) => fields.get(name).cloned(),
        _ => None, // Other filter types don't directly evaluate to values
    }
}

/// Compare two DBValues for equality
fn values_equal(left: &DBValue, right: &DBValue) -> bool {
    match (left, right) {
        (DBValue::String(l), DBValue::String(r)) => l == r,
        (DBValue::Number(l), DBValue::Number(r)) => l == r,
        (DBValue::Timestamp(l), DBValue::Timestamp(r)) => l == r,
        _ => false, // Different types are not equal
    }
}
