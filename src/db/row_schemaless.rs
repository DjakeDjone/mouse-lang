use crate::db::{query_engine, DBValue, DBValueType, FilterEntity};
use std::collections::{HashMap, HashSet};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

pub struct Settings {
    pub base_path: String,
}

pub struct TableRowSchemaless {
    pub settings: Settings,
    pub primary_key: String,
    pub known_columns: HashSet<(String, DBValueType)>, // schemaless, so it's possible to insert data that is not in this schema
}

impl TableRowSchemaless {
    pub async fn new(pk: String, settings: Settings) -> Self {
        // create file
        if !std::path::Path::new(&settings.base_path).exists() {
            tokio::fs::create_dir_all(&settings.base_path)
                .await
                .expect("Failed to create directory");
        }
        Self {
            settings,
            primary_key: pk,
            known_columns: HashSet::new(),
        }
    }

    pub async fn insert(&mut self, data: HashMap<String, DBValue>) {
        for (k, v) in &data {
            self.known_columns.insert((k.to_owned(), v.vtype()));
        }

        // TODO: insert into cache
        // add to file

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/{}", self.settings.base_path, self.primary_key))
            .await
            .unwrap();

        // Serialize with bincode (2.0 API)
        let config = bincode::config::standard();
        let bytes = bincode::encode_to_vec(&data, config).unwrap();
        // Write length prefix (4 bytes for u32)
        let len = bytes.len() as u32;
        file.write_all(&len.to_le_bytes()).await.unwrap();
        // Write the actual data
        file.write_all(&bytes).await.unwrap();
        file.flush().await.unwrap();
    }

    pub async fn drop(&mut self) {
        // TODO: delete cache
        // delete the file
        tokio::fs::remove_file(format!("{}/{}", self.settings.base_path, self.primary_key))
            .await
            .expect("Failed to remove file");
    }

    pub async fn truncate(&mut self) {
        // TODO: delete cache
        // remove all rows
        let path = format!("{}/{}", self.settings.base_path, self.primary_key);
        let _ = tokio::fs::remove_file(&path).await; // Ignore error if file doesn't exist
        tokio::fs::File::create(&path)
            .await
            .expect("Failed to create file");
    }

    pub async fn query(&self, query: FilterEntity) -> Vec<HashMap<String, DBValue>> {
        // let _necessary_fields = query_engine::pre_select(&query).unwrap_or(
        //     self.known_columns
        //         .iter()
        //         .map(|col| PreSelectedField::from_column(col.clone()))
        //         .collect(),
        // );

        // later do preselect with partitioning etc.
        // for now just go though every row
        let mut result = Vec::new();
        let file = OpenOptions::new()
            .read(true)
            .open(format!("{}/{}", self.settings.base_path, self.primary_key))
            .await
            .unwrap();
        let mut reader = BufReader::new(file);

        // Read length-prefixed binary records
        loop {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes).await {
                Ok(_) => {}
                Err(_) => break, // EOF or error
            }
            let len = u32::from_le_bytes(len_bytes) as usize;

            let mut buffer = vec![0u8; len];
            reader.read_exact(&mut buffer).await.unwrap();

            let config = bincode::config::standard();
            let (row, _): (HashMap<String, DBValue>, usize) =
                bincode::decode_from_slice(&buffer, config).unwrap();
            if query_engine::execute_query(&query, &row) {
                result.push(row);
            }
        }
        result
    }

    /// returns false if file not exists
    pub async fn is_empty(&self) -> bool {
        let file = OpenOptions::new()
            .read(true)
            .open(format!("{}/{}", self.settings.base_path, self.primary_key))
            .await;

        match file {
            Ok(f) => {
                let mut reader = BufReader::new(f);
                reader
                    .fill_buf()
                    .await
                    .map(|buf| buf.is_empty())
                    .unwrap_or(true)
            }
            Err(_) => true, // If file doesn't exist, consider it empty
        }
    }

    pub async fn size(&self) -> usize {
        let file = OpenOptions::new()
            .read(true)
            .open(format!("{}/{}", self.settings.base_path, self.primary_key))
            .await
            .unwrap();
        let mut reader = BufReader::new(file);

        let mut count = 0;
        loop {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes).await {
                Ok(_) => {}
                Err(_) => break, // EOF or error
            }
            let len = u32::from_le_bytes(len_bytes) as usize;

            let mut buffer = vec![0u8; len];
            reader.read_exact(&mut buffer).await.unwrap();

            count += 1;
        }
        count
    }
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert() {
        let mut table = TableRowSchemaless::new(
            "id".to_string(),
            Settings {
                base_path: "test".to_string(),
            },
        )
        .await;
        table.truncate().await;
        table
            .insert(HashMap::from([
                ("id".to_string(), DBValue::Number(1.0)),
                ("column1".to_string(), DBValue::String("value1".to_string())),
                ("column2".to_string(), DBValue::String("value2".to_string())),
            ]))
            .await;

        table
            .insert(HashMap::from([
                ("id".to_string(), DBValue::Number(2.0)),
                ("column1".to_string(), DBValue::String("value3".to_string())),
                ("column2".to_string(), DBValue::String("value4".to_string())),
            ]))
            .await;

        assert!(table
            .known_columns
            .contains(&("column1".to_string(), DBValueType::String)));

        table.drop().await;
    }

    #[tokio::test]
    async fn test_query() {
        let mut table = TableRowSchemaless::new(
            "id".to_string(),
            Settings {
                base_path: "test_db/test_query".to_string(),
            },
        )
        .await;

        table.truncate().await;

        table
            .insert(HashMap::from([
                ("id".to_string(), DBValue::Number(1.0)),
                ("column1".to_string(), DBValue::String("value1".to_string())),
                ("column2".to_string(), DBValue::String("value2".to_string())),
            ]))
            .await;

        table
            .insert(HashMap::from([
                ("id".to_string(), DBValue::Number(2.0)),
                ("column1".to_string(), DBValue::String("value3".to_string())),
                ("column2".to_string(), DBValue::String("value4".to_string())),
            ]))
            .await;

        let query = FilterEntity::And(
            Box::new(FilterEntity::Equals(
                Box::new(FilterEntity::Column("column1".to_string())),
                Box::new(FilterEntity::Value(DBValue::String("value1".to_string()))),
            )),
            Box::new(FilterEntity::Equals(
                Box::new(FilterEntity::Column("column2".to_string())),
                Box::new(FilterEntity::Value(DBValue::String("value2".to_string()))),
            )),
        );
        let rows = table.query(query).await;
        println!("result: {:?}", rows);
        assert_eq!(rows.len(), 1);

        let _result = rows[0].clone();
        table.drop().await;

        // assert_eq!(result.get("id").unwrap(), &DBValue::Number(1.0));
    }

    async fn insert_test_data_if_not_exists(table: &mut TableRowSchemaless) {
        if table.is_empty().await {
            for i in 0..10_000 {
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
        }
    }

    #[tokio::test]
    async fn test_query_performance() {
        let mut table = TableRowSchemaless::new(
            "id".to_string(),
            Settings {
                base_path: "test_db/test_performance".to_string(),
            },
        )
        .await;

        insert_test_data_if_not_exists(&mut table).await;

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
                Box::new(FilterEntity::Value(DBValue::String("value2".to_string()))),
            )),
        );
        let rows = table.query(query).await;
        println!("result: {:?}", rows);
        assert_eq!(rows.len(), 2);
        // assert_eq!(result.get("id").unwrap(), &DBValue::Number(1.0));
    }

    #[tokio::test]
    async fn test_fuzzy_search() {
        let mut table = TableRowSchemaless::new(
            "id".to_string(),
            Settings {
                base_path: "test_db/test_fuzzy_search".to_string(),
            },
        )
        .await;

        table.truncate().await;

        insert_test_data_if_not_exists(&mut table).await;

        table
            .insert(HashMap::from([
                ("id".to_string(), DBValue::Number(1.0)),
                ("column1".to_string(), DBValue::String("Buch".to_string())),
                ("column2".to_string(), DBValue::String("value2".to_string())),
            ]))
            .await;

        let query = FilterEntity::FuzzyMatch(
            Box::new(FilterEntity::Column("column1".to_string())),
            Box::new(FilterEntity::Value(DBValue::String("buche".to_string()))),
            2, // treshold
        );
        let rows = table.query(query).await;
        println!("result: {:?}", rows);
        assert_eq!(rows.len(), 1);

        table.drop().await;
    }

    #[tokio::test]
    async fn test_serialization_comparison() {
        let sample_data = HashMap::from([
            ("id".to_string(), DBValue::Number(12345.0)),
            (
                "column1".to_string(),
                DBValue::String("test_value_with_some_length".to_string()),
            ),
            (
                "column2".to_string(),
                DBValue::String("another_test_value".to_string()),
            ),
            ("date".to_string(), DBValue::Timestamp(1672531200)),
            ("amount".to_string(), DBValue::Number(999.99)),
        ]);

        let iterations = 10000;

        // Test JSON serialization
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let json = serde_json::to_string(&sample_data).unwrap();
            let _: HashMap<String, DBValue> = serde_json::from_str(&json).unwrap();
        }
        let json_duration = start.elapsed();

        // Test bincode serialization
        let config = bincode::config::standard();
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let bytes = bincode::encode_to_vec(&sample_data, config).unwrap();
            let (_decoded, _): (HashMap<String, DBValue>, usize) =
                bincode::decode_from_slice(&bytes, config).unwrap();
        }
        let bincode_duration = start.elapsed();

        println!("\n=== Serialization Performance Comparison ===");
        println!("Iterations: {}", iterations);
        println!(
            "JSON:    {:?} ({:.2} µs/iter)",
            json_duration,
            json_duration.as_micros() as f64 / iterations as f64
        );
        println!(
            "Bincode: {:?} ({:.2} µs/iter)",
            bincode_duration,
            bincode_duration.as_micros() as f64 / iterations as f64
        );
        println!(
            "Speedup: {:.2}x faster",
            json_duration.as_secs_f64() / bincode_duration.as_secs_f64()
        );

        // Verify bincode is faster
        assert!(
            bincode_duration < json_duration,
            "Bincode should be faster than JSON"
        );
    }

    #[tokio::test]
    async fn test_debug_simple_insert_read() {
        let mut table = TableRowSchemaless::new(
            "id".to_string(),
            Settings {
                base_path: "test_db/test_debug".to_string(),
            },
        )
        .await;

        table.truncate().await;

        // Insert records in a loop - using exact same format as insert_test_data_if_not_exists
        let num_records = 100000;
        println!("Inserting {} records...", num_records);
        for i in 0..num_records {
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
                        DBValue::Timestamp(1672531200 + (i as i64) * 86400),
                    ),
                    ("amount".to_string(), DBValue::Number((i * 2) as f64)),
                ]))
                .await;

            if i % 1000 == 0 {
                println!("Inserted {} records", i);
            }
        }

        println!("Querying back all records...");
        let query = FilterEntity::GreaterThan(
            Box::new(FilterEntity::Column("id".to_string())),
            Box::new(FilterEntity::Value(DBValue::Number(-1.0))),
        );
        let rows = table.query(query).await;
        println!("Successfully queried {} records", rows.len());
        assert_eq!(rows.len(), num_records);

        table.drop().await;
    }

    #[tokio::test]
    async fn test_size() {
        let mut table = TableRowSchemaless::new(
            "id".to_string(),
            Settings {
                base_path: "test_db/test_debug".to_string(),
            },
        )
        .await;

        table.truncate().await;

        // Insert records in a loop - using exact same format as insert_test_data_if_not_exists
        let num_records = 10_000;
        for i in 0..num_records {
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
                        DBValue::Timestamp(1672531200 + (i as i64) * 86400),
                    ),
                    ("amount".to_string(), DBValue::Number((i * 2) as f64)),
                ]))
                .await;
        }

        let size = table.size().await;
        println!("Table size: {}", size);
        assert_eq!(size, num_records);

        table.drop().await;
    }
}
