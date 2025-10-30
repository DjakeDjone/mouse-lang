use criterion::{criterion_group, criterion_main, Criterion};
use mouse_lang::db::row_schemaless::{Settings, TableRowSchemaless};
use mouse_lang::db::{DBValue, FilterEntity};
use std::collections::HashMap;
use std::hint::black_box;

async fn setup_test_table() -> TableRowSchemaless {
    let mut table = TableRowSchemaless::new(
        "id".to_string(),
        Settings {
            base_path: "test_db/benchmark".to_string(),
        },
    )
    .await;

    // Insert test data if the table is empty
    if table.is_empty().await {
        println!("Inserting 100,000 rows for benchmark...");
        for i in 0..100000 {
            table
                .insert(HashMap::from([
                    ("id".to_string(), DBValue::Number(i as f64)),
                    (
                        "column1".to_string(),
                        DBValue::String(format!("value{}", i)),
                    ),
                    (
                        "column2".to_string(),
                        DBValue::String(format!("value{}- {}", i, i)),
                    ),
                    (
                        "date".to_string(),
                        DBValue::Timestamp(1672531200 + i * 86400),
                    ),
                    ("amount".to_string(), DBValue::Number((i * 2) as f64)),
                ]))
                .await;
        }
        println!("Test data inserted successfully!");
    }

    table
}

async fn setup_test_table_with_indexes() -> TableRowSchemaless {
    let mut table = TableRowSchemaless::new(
        "id".to_string(),
        Settings {
            base_path: "test_db/benchmark_indexed".to_string(),
        },
    )
    .await;

    // Insert test data if the table is empty
    if table.is_empty().await {
        println!("Inserting 100,000 rows for indexed benchmark...");
        for i in 0..100000 {
            table
                .insert(HashMap::from([
                    ("id".to_string(), DBValue::Number(i as f64)),
                    (
                        "column1".to_string(),
                        DBValue::String(format!("value{}", i)),
                    ),
                    (
                        "column2".to_string(),
                        DBValue::String(format!("value{}- {}", i, i)),
                    ),
                    (
                        "date".to_string(),
                        DBValue::Timestamp(1672531200 + i * 86400),
                    ),
                    ("amount".to_string(), DBValue::Number((i * 2) as f64)),
                ]))
                .await;
        }
        println!("Test data inserted successfully!");

        // Create indexes on commonly queried columns
        println!("Creating indexes...");
        table.create_index("column1").await;
        table.create_index("column2").await;
        table.create_index("amount").await;
        table.create_index("date").await;
        println!("Indexes created: {:?}", table.list_indexes());
    } else {
        // Ensure indexes exist even if data was already present
        let indexes = table.list_indexes();
        if !indexes.contains(&"column1".to_string()) {
            println!("Creating indexes on existing data...");
            table.create_index("column1").await;
            table.create_index("column2").await;
            table.create_index("amount").await;
            table.create_index("date").await;
            println!("Indexes created: {:?}", table.list_indexes());
        }
    }

    table
}

fn query_simple_equals(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once
    let table = runtime.block_on(setup_test_table());

    c.bench_function("query_simple_equals_no_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::Equals(
                Box::new(FilterEntity::Column("column1".to_string())),
                Box::new(FilterEntity::Value(DBValue::String(
                    "value5000".to_string(),
                ))),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

fn query_simple_equals_indexed(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once with indexes
    let table = runtime.block_on(setup_test_table_with_indexes());

    c.bench_function("query_simple_equals_with_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::Equals(
                Box::new(FilterEntity::Column("column1".to_string())),
                Box::new(FilterEntity::Value(DBValue::String(
                    "value5000".to_string(),
                ))),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

fn query_or_multiple_conditions(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once
    let table = runtime.block_on(setup_test_table());

    c.bench_function("query_or_multiple_conditions_no_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::Or(
                Box::new(FilterEntity::Or(
                    Box::new(FilterEntity::Equals(
                        Box::new(FilterEntity::Column("column1".to_string())),
                        Box::new(FilterEntity::Value(DBValue::String(
                            "value5000".to_string(),
                        ))),
                    )),
                    Box::new(FilterEntity::Equals(
                        Box::new(FilterEntity::Column("amount".to_string())),
                        Box::new(FilterEntity::Value(DBValue::Number(2.0))),
                    )),
                )),
                Box::new(FilterEntity::Equals(
                    Box::new(FilterEntity::Column("column2".to_string())),
                    Box::new(FilterEntity::Value(DBValue::String(
                        "value2- 2".to_string(),
                    ))),
                )),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

fn query_or_multiple_conditions_indexed(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once with indexes
    let table = runtime.block_on(setup_test_table_with_indexes());

    c.bench_function("query_or_multiple_conditions_with_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::Or(
                Box::new(FilterEntity::Or(
                    Box::new(FilterEntity::Equals(
                        Box::new(FilterEntity::Column("column1".to_string())),
                        Box::new(FilterEntity::Value(DBValue::String(
                            "value5000".to_string(),
                        ))),
                    )),
                    Box::new(FilterEntity::Equals(
                        Box::new(FilterEntity::Column("amount".to_string())),
                        Box::new(FilterEntity::Value(DBValue::Number(2.0))),
                    )),
                )),
                Box::new(FilterEntity::Equals(
                    Box::new(FilterEntity::Column("column2".to_string())),
                    Box::new(FilterEntity::Value(DBValue::String(
                        "value2- 2".to_string(),
                    ))),
                )),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

fn query_and_conditions(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once
    let table = runtime.block_on(setup_test_table());

    c.bench_function("query_and_conditions_no_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::And(
                Box::new(FilterEntity::GreaterThan(
                    Box::new(FilterEntity::Column("amount".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Number(1000000.0))),
                )),
                Box::new(FilterEntity::LessThan(
                    Box::new(FilterEntity::Column("amount".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Number(1001000.0))),
                )),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

fn query_and_conditions_indexed(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once with indexes
    let table = runtime.block_on(setup_test_table_with_indexes());

    c.bench_function("query_and_conditions_with_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::And(
                Box::new(FilterEntity::GreaterThan(
                    Box::new(FilterEntity::Column("amount".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Number(1000000.0))),
                )),
                Box::new(FilterEntity::LessThan(
                    Box::new(FilterEntity::Column("amount".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Number(1001000.0))),
                )),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

fn query_timestamp_range(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once
    let table = runtime.block_on(setup_test_table());

    c.bench_function("query_timestamp_range_no_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::And(
                Box::new(FilterEntity::GreaterThan(
                    Box::new(FilterEntity::Column("date".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Timestamp(
                        1672531200 + 50000 * 86400,
                    ))),
                )),
                Box::new(FilterEntity::LessThan(
                    Box::new(FilterEntity::Column("date".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Timestamp(
                        1672531200 + 50100 * 86400,
                    ))),
                )),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

fn query_timestamp_range_indexed(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once with indexes
    let table = runtime.block_on(setup_test_table_with_indexes());

    c.bench_function("query_timestamp_range_with_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::And(
                Box::new(FilterEntity::GreaterThan(
                    Box::new(FilterEntity::Column("date".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Timestamp(
                        1672531200 + 50000 * 86400,
                    ))),
                )),
                Box::new(FilterEntity::LessThan(
                    Box::new(FilterEntity::Column("date".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Timestamp(
                        1672531200 + 50100 * 86400,
                    ))),
                )),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

fn query_complex_nested(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once
    let table = runtime.block_on(setup_test_table());

    c.bench_function("query_complex_nested_no_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::And(
                Box::new(FilterEntity::Or(
                    Box::new(FilterEntity::Equals(
                        Box::new(FilterEntity::Column("column1".to_string())),
                        Box::new(FilterEntity::Value(DBValue::String(
                            "value1000".to_string(),
                        ))),
                    )),
                    Box::new(FilterEntity::Equals(
                        Box::new(FilterEntity::Column("column1".to_string())),
                        Box::new(FilterEntity::Value(DBValue::String(
                            "value2000".to_string(),
                        ))),
                    )),
                )),
                Box::new(FilterEntity::GreaterThan(
                    Box::new(FilterEntity::Column("amount".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Number(1000.0))),
                )),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

fn query_complex_nested_indexed(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup once with indexes
    let table = runtime.block_on(setup_test_table_with_indexes());

    c.bench_function("query_complex_nested_with_index", |b| {
        b.to_async(&runtime).iter(|| async {
            let query = FilterEntity::And(
                Box::new(FilterEntity::Or(
                    Box::new(FilterEntity::Equals(
                        Box::new(FilterEntity::Column("column1".to_string())),
                        Box::new(FilterEntity::Value(DBValue::String(
                            "value1000".to_string(),
                        ))),
                    )),
                    Box::new(FilterEntity::Equals(
                        Box::new(FilterEntity::Column("column1".to_string())),
                        Box::new(FilterEntity::Value(DBValue::String(
                            "value2000".to_string(),
                        ))),
                    )),
                )),
                Box::new(FilterEntity::GreaterThan(
                    Box::new(FilterEntity::Column("amount".to_string())),
                    Box::new(FilterEntity::Value(DBValue::Number(1000.0))),
                )),
            );

            let rows = table.query(black_box(query)).await;
            black_box(rows)
        });
    });
}

criterion_group!(
    benches,
    query_simple_equals,
    query_simple_equals_indexed,
    query_or_multiple_conditions,
    query_or_multiple_conditions_indexed,
    query_and_conditions,
    query_and_conditions_indexed,
    query_timestamp_range,
    query_timestamp_range_indexed,
    query_complex_nested,
    query_complex_nested_indexed
);
criterion_main!(benches);
