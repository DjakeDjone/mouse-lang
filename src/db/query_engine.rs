use std::collections::HashMap;

use crate::db::{DBValue, DBValueType, FilterEntity};
use strsim;

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
    let mut columns: HashMap<String, PreSelectedField> = HashMap::new();

    collect_columns(query, &mut columns);

    if columns.is_empty() {
        None
    } else {
        Some(columns.into_values().collect())
    }
}

/// Recursively collect all columns from a filter entity
fn collect_columns(filter: &FilterEntity, columns: &mut HashMap<String, PreSelectedField>) {
    match filter {
        FilterEntity::Column(name) => {
            // Add column if not already present
            columns
                .entry(name.clone())
                .or_insert_with(|| PreSelectedField {
                    name: name.clone(),
                    kind: DBValueType::String, // Default type
                    range: None,
                });
        }
        FilterEntity::Equals(left, right) => {
            // Try to infer type from value comparisons
            infer_from_comparison(left, right, columns);
            collect_columns(left, columns);
            collect_columns(right, columns);
        }
        FilterEntity::GreaterThan(left, right) | FilterEntity::LessThan(left, right) => {
            // Numeric comparisons - infer Number type
            infer_numeric_type(left, columns);
            infer_numeric_type(right, columns);
            collect_columns(left, columns);
            collect_columns(right, columns);
        }
        FilterEntity::FuzzyMatch(left, right, _) => {
            // String comparisons - infer String type
            infer_string_type(left, columns);
            infer_string_type(right, columns);
            collect_columns(left, columns);
            collect_columns(right, columns);
        }
        FilterEntity::Not(inner) => {
            collect_columns(inner, columns);
        }
        FilterEntity::And(left, right)
        | FilterEntity::Or(left, right)
        | FilterEntity::Xor(left, right) => {
            collect_columns(left, columns);
            collect_columns(right, columns);
        }
        FilterEntity::Value(_) => {
            // Values don't contribute columns
        }
        FilterEntity::Null => {
            // Null doesn't contribute columns
        }
    }
}

/// Infer column type from comparison with a value
pub fn infer_from_comparison(
    left: &FilterEntity,
    right: &FilterEntity,
    columns: &mut HashMap<String, PreSelectedField>,
) {
    match (left, right) {
        (FilterEntity::Column(name), FilterEntity::Value(val))
        | (FilterEntity::Value(val), FilterEntity::Column(name)) => {
            columns
                .entry(name.clone())
                .and_modify(|field| {
                    field.kind = val.vtype();
                })
                .or_insert_with(|| PreSelectedField {
                    name: name.clone(),
                    kind: val.vtype(),
                    range: None,
                });
        }
        _ => {}
    }
}

/// Infer that a column is numeric type
pub fn infer_numeric_type(filter: &FilterEntity, columns: &mut HashMap<String, PreSelectedField>) {
    if let FilterEntity::Column(name) = filter {
        columns
            .entry(name.clone())
            .and_modify(|field| {
                field.kind = DBValueType::Number;
            })
            .or_insert_with(|| PreSelectedField {
                name: name.clone(),
                kind: DBValueType::Number,
                range: None,
            });
    }
}

/// Infer that a column is string type
pub fn infer_string_type(filter: &FilterEntity, columns: &mut HashMap<String, PreSelectedField>) {
    if let FilterEntity::Column(name) = filter {
        columns
            .entry(name.clone())
            .and_modify(|field| {
                field.kind = DBValueType::String;
            })
            .or_insert_with(|| PreSelectedField {
                name: name.clone(),
                kind: DBValueType::String,
                range: None,
            });
    }
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
        FilterEntity::GreaterThan(left, right) => {
            match (
                evaluate_to_value(left, fields),
                evaluate_to_value(right, fields),
            ) {
                (Some(DBValue::Number(l)), Some(DBValue::Number(r))) => l > r,
                _ => false,
            }
        }
        FilterEntity::LessThan(left, right) => {
            match (
                evaluate_to_value(left, fields),
                evaluate_to_value(right, fields),
            ) {
                (Some(DBValue::Number(l)), Some(DBValue::Number(r))) => l < r,
                _ => false,
            }
        }
        FilterEntity::FuzzyMatch(left, right, threshold) => {
            match (
                evaluate_to_value(left, fields),
                evaluate_to_value(right, fields),
            ) {
                (Some(DBValue::String(l)), Some(DBValue::String(r))) => {
                    let distance = strsim::levenshtein(&l, &r);
                    distance <= *threshold as usize
                }
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
        FilterEntity::Null => false,
    }
}

/// Evaluate a filter entity to a concrete value
fn evaluate_to_value(filter: &FilterEntity, fields: &HashMap<String, DBValue>) -> Option<DBValue> {
    match filter {
        FilterEntity::Value(val) => Some(val.clone()),
        FilterEntity::Column(name) => Some(fields.get(name).cloned().unwrap_or(DBValue::Null)),
        _ => None, // Other filter types don't directly evaluate to values
    }
}

/// Compare two DBValues for equality
fn values_equal(left: &DBValue, right: &DBValue) -> bool {
    match (left, right) {
        (DBValue::String(l), DBValue::String(r)) => l == r,
        (DBValue::Number(l), DBValue::Number(r)) => l == r,
        (DBValue::Timestamp(l), DBValue::Timestamp(r)) => l == r,
        (DBValue::Null, DBValue::Null) => true,
        (DBValue::Null, _) => false,
        (_, DBValue::Null) => false,
        _ => false, // Different types are not equal
    }
}
