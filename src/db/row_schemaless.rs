use crate::db::{query_engine, DBValue, DBValueType, FilterEntity};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

#[derive(Clone)]
pub struct Settings {
    pub base_path: String,
}

pub struct TableRowSchemaless {
    pub settings: Settings,
    pub primary_key: String,
    pub known_columns: HashSet<(String, DBValueType)>, // schemaless, so it's possible to insert data that is not in this schema
    // Indexes: column_name -> (indexed_value -> Vec<row_id>)
    indexes: Arc<RwLock<HashMap<String, BTreeMap<String, Vec<u64>>>>>,
    next_row_id: Arc<RwLock<u64>>,
}

impl TableRowSchemaless {
    pub async fn new(pk: String, settings: Settings) -> Self {
        // create file
        if !std::path::Path::new(&settings.base_path).exists() {
            tokio::fs::create_dir_all(&settings.base_path)
                .await
                .expect("Failed to create directory");
        }

        let mut table = Self {
            settings,
            primary_key: pk,
            known_columns: HashSet::new(),
            indexes: Arc::new(RwLock::new(HashMap::new())),
            next_row_id: Arc::new(RwLock::new(0)),
        };

        // Load indexes from disk if they exist
        table.load_indexes().await;

        // Initialize next_row_id by counting existing rows
        let row_count = table.size().await;
        *table.next_row_id.write().unwrap() = row_count as u64;

        table
    }

    fn value_to_index_key(value: &DBValue) -> String {
        match value {
            DBValue::String(s) => format!("s:{}", s),
            DBValue::Number(n) => format!("n:{:020.6}", n),
            DBValue::Timestamp(t) => format!("t:{:020}", t),
            DBValue::Null => "null".to_string(),
        }
    }

    /// Create an index on a specified column
    pub async fn create_index(&mut self, column: &str) {
        let mut indexes = self.indexes.write().unwrap();

        // Check if index already exists
        if indexes.contains_key(column) {
            return;
        }

        // Create new index
        let mut index: BTreeMap<String, Vec<u64>> = BTreeMap::new();

        // Read all rows and build index
        let file_result = OpenOptions::new()
            .read(true)
            .open(format!("{}/{}", self.settings.base_path, self.primary_key))
            .await;

        if let Ok(file) = file_result {
            let mut reader = BufReader::new(file);
            let mut row_id = 0u64;

            loop {
                let mut len_bytes = [0u8; 4];
                match reader.read_exact(&mut len_bytes).await {
                    Ok(_) => {}
                    Err(_) => break,
                }
                let len = u32::from_le_bytes(len_bytes) as usize;

                let mut buffer = vec![0u8; len];
                if reader.read_exact(&mut buffer).await.is_err() {
                    break;
                }

                let config = bincode::config::standard();
                if let Ok((row, _)) =
                    bincode::decode_from_slice::<HashMap<String, DBValue>, _>(&buffer, config)
                {
                    if let Some(value) = row.get(column) {
                        let key = Self::value_to_index_key(value);
                        index.entry(key).or_insert_with(Vec::new).push(row_id);
                    }
                }

                row_id += 1;
            }
        }

        indexes.insert(column.to_string(), index);
        drop(indexes); // Release lock before async operation

        // Persist indexes
        self.save_indexes().await;
    }

    /// Drop an index on a specified column
    pub async fn drop_index(&mut self, column: &str) {
        let mut indexes = self.indexes.write().unwrap();
        indexes.remove(column);
        drop(indexes); // Release lock before async operation

        // Persist indexes
        self.save_indexes().await;
    }

    /// List all indexed columns
    pub fn list_indexes(&self) -> Vec<String> {
        let indexes = self.indexes.read().unwrap();
        indexes.keys().cloned().collect()
    }

    async fn save_indexes(&self) {
        let indexes = self.indexes.read().unwrap();
        let index_path = format!("{}/{}.idx", self.settings.base_path, self.primary_key);

        let config = bincode::config::standard();
        if let Ok(bytes) = bincode::encode_to_vec(&*indexes, config) {
            if let Ok(mut file) = tokio::fs::File::create(&index_path).await {
                let _ = file.write_all(&bytes).await;
            }
        }
    }

    async fn load_indexes(&mut self) {
        let index_path = format!("{}/{}.idx", self.settings.base_path, self.primary_key);

        if let Ok(mut file) = tokio::fs::File::open(&index_path).await {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).await.is_ok() {
                let config = bincode::config::standard();
                if let Ok((loaded_indexes, _)) = bincode::decode_from_slice::<
                    HashMap<String, BTreeMap<String, Vec<u64>>>,
                    _,
                >(&buffer, config)
                {
                    *self.indexes.write().unwrap() = loaded_indexes;
                }
            }
        }
    }

    pub async fn insert(&mut self, data: HashMap<String, DBValue>) {
        for (k, v) in &data {
            self.known_columns.insert((k.to_owned(), v.vtype()));
        }

        // Get the row ID for this insert
        let row_id = {
            let mut next_id = self.next_row_id.write().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Update indexes
        {
            let mut indexes = self.indexes.write().unwrap();
            for (column, index) in indexes.iter_mut() {
                if let Some(value) = data.get(column) {
                    let key = Self::value_to_index_key(value);
                    index.entry(key).or_insert_with(Vec::new).push(row_id);
                }
            }
        }

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

        // Persist indexes if any exist
        if !self.indexes.read().unwrap().is_empty() {
            self.save_indexes().await;
        }
    }

    pub async fn drop(&mut self) {
        // Clear indexes
        {
            let mut indexes = self.indexes.write().unwrap();
            indexes.clear();
        }

        // delete the file
        tokio::fs::remove_file(format!("{}/{}", self.settings.base_path, self.primary_key))
            .await
            .expect("Failed to remove file");

        // Delete index file
        let index_path = format!("{}/{}.idx", self.settings.base_path, self.primary_key);
        let _ = tokio::fs::remove_file(&index_path).await;

        // Reset row counter
        *self.next_row_id.write().unwrap() = 0;
    }

    pub async fn truncate(&mut self) {
        // Clear indexes but keep index definitions
        {
            let mut indexes = self.indexes.write().unwrap();
            for index in indexes.values_mut() {
                index.clear();
            }
        }

        // remove all rows
        let path = format!("{}/{}", self.settings.base_path, self.primary_key);
        let _ = tokio::fs::remove_file(&path).await; // Ignore error if file doesn't exist
        tokio::fs::File::create(&path)
            .await
            .expect("Failed to create file");

        // Reset row counter
        *self.next_row_id.write().unwrap() = 0;

        // Persist empty indexes
        if !self.indexes.read().unwrap().is_empty() {
            self.save_indexes().await;
        }
    }

    pub async fn query(&self, query: FilterEntity) -> Vec<HashMap<String, DBValue>> {
        // Try to use index if available for simple equality queries
        if let Some((_column, _value, row_ids)) = self.try_use_index(&query) {
            return self.query_by_row_ids(&row_ids, &query).await;
        }

        // Fall back to full table scan
        self.query_full_scan(query).await
    }

    fn try_use_index(&self, query: &FilterEntity) -> Option<(String, DBValue, Vec<u64>)> {
        // Check for simple equality: Equals(Column(name), Value(val)) or Equals(Value(val), Column(name))
        if let FilterEntity::Equals(left, right) = query {
            let indexes = self.indexes.read().unwrap();

            match (left.as_ref(), right.as_ref()) {
                (FilterEntity::Column(col), FilterEntity::Value(val)) => {
                    if let Some(index) = indexes.get(col) {
                        let key = Self::value_to_index_key(val);
                        if let Some(row_ids) = index.get(&key) {
                            return Some((col.clone(), val.clone(), row_ids.clone()));
                        }
                    }
                }
                (FilterEntity::Value(val), FilterEntity::Column(col)) => {
                    if let Some(index) = indexes.get(col) {
                        let key = Self::value_to_index_key(val);
                        if let Some(row_ids) = index.get(&key) {
                            return Some((col.clone(), val.clone(), row_ids.clone()));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    async fn query_by_row_ids(
        &self,
        row_ids: &[u64],
        query: &FilterEntity,
    ) -> Vec<HashMap<String, DBValue>> {
        let mut result = Vec::new();

        let file = OpenOptions::new()
            .read(true)
            .open(format!("{}/{}", self.settings.base_path, self.primary_key))
            .await
            .unwrap();
        let mut reader = BufReader::new(file);

        let mut current_row_id = 0u64;
        let row_id_set: HashSet<u64> = row_ids.iter().copied().collect();

        loop {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes).await {
                Ok(_) => {}
                Err(_) => break,
            }
            let len = u32::from_le_bytes(len_bytes) as usize;

            let mut buffer = vec![0u8; len];
            if reader.read_exact(&mut buffer).await.is_err() {
                break;
            }

            if row_id_set.contains(&current_row_id) {
                let config = bincode::config::standard();
                if let Ok((row, _)) =
                    bincode::decode_from_slice::<HashMap<String, DBValue>, _>(&buffer, config)
                {
                    if query_engine::execute_query(query, &row) {
                        result.push(row);
                    }
                }
            }

            current_row_id += 1;
        }
        result
    }

    async fn query_full_scan(&self, query: FilterEntity) -> Vec<HashMap<String, DBValue>> {
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
            if reader.read_exact(&mut buffer).await.is_err() {
                break;
            }

            let config = bincode::config::standard();
            if let Ok((row, _)) =
                bincode::decode_from_slice::<HashMap<String, DBValue>, _>(&buffer, config)
            {
                if query_engine::execute_query(&query, &row) {
                    result.push(row);
                }
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
        let file_result = OpenOptions::new()
            .read(true)
            .open(format!("{}/{}", self.settings.base_path, self.primary_key))
            .await;

        let file = match file_result {
            Ok(f) => f,
            Err(_) => return 0, // File doesn't exist, so size is 0
        };

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
            if reader.read_exact(&mut buffer).await.is_err() {
                break;
            }

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
        println!("JSON duration: {:?}", json_duration);

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
    }

    #[tokio::test]
    async fn test_create_index() {
        let settings = Settings {
            base_path: "test_db/test_create_index".to_string(),
        };

        let _ = tokio::fs::remove_dir_all(&settings.base_path).await;

        let mut table = TableRowSchemaless::new("test_table_idx".to_string(), settings).await;
        table.truncate().await;

        // Insert test data
        for i in 0..10 {
            let mut data = HashMap::new();
            data.insert("id".to_string(), DBValue::Number(i as f64));
            data.insert("name".to_string(), DBValue::String(format!("Person{}", i)));
            table.insert(data).await;
        }

        // Create index on 'name' column
        table.create_index("name").await;

        // Verify index exists
        let indexes = table.list_indexes();
        assert!(indexes.contains(&"name".to_string()));

        table.drop().await;
        let _ = tokio::fs::remove_dir_all("test_db/test_create_index").await;
    }

    #[tokio::test]
    async fn test_query_with_index() {
        let settings = Settings {
            base_path: "test_db/test_query_with_index".to_string(),
        };

        let _ = tokio::fs::remove_dir_all(&settings.base_path).await;

        let mut table = TableRowSchemaless::new("test_table_idx2".to_string(), settings).await;
        table.truncate().await;

        // Insert test data
        for i in 0..20 {
            let mut data = HashMap::new();
            data.insert("id".to_string(), DBValue::Number(i as f64));
            data.insert(
                "status".to_string(),
                DBValue::String(if i % 2 == 0 {
                    "active".to_string()
                } else {
                    "inactive".to_string()
                }),
            );
            table.insert(data).await;
        }

        // Create index on 'status' column
        table.create_index("status").await;

        // Query using index
        let query = FilterEntity::Equals(
            Box::new(FilterEntity::Column("status".to_string())),
            Box::new(FilterEntity::Value(DBValue::String("active".to_string()))),
        );

        let result = table.query(query).await;
        assert_eq!(result.len(), 10); // Should match 10 active entries

        table.drop().await;
        let _ = tokio::fs::remove_dir_all("test_db/test_query_with_index").await;
    }

    #[tokio::test]
    async fn test_index_persistence() {
        let settings = Settings {
            base_path: "test_db/test_index_persistence".to_string(),
        };

        let _ = tokio::fs::remove_dir_all(&settings.base_path).await;

        // Create table and index
        {
            let mut table =
                TableRowSchemaless::new("test_table_persist".to_string(), settings.clone()).await;
            table.truncate().await;

            // Insert test data
            for i in 0..5 {
                let mut data = HashMap::new();
                data.insert("id".to_string(), DBValue::Number(i as f64));
                data.insert(
                    "category".to_string(),
                    DBValue::String(format!("cat{}", i % 3)),
                );
                table.insert(data).await;
            }

            // Create index
            table.create_index("category").await;

            // Don't call drop - let it go out of scope to test persistence
        }

        // Reopen table and verify index still exists
        {
            let table =
                TableRowSchemaless::new("test_table_persist".to_string(), settings.clone()).await;
            let indexes = table.list_indexes();
            assert!(indexes.contains(&"category".to_string()));

            // Query should still use the persisted index
            let query = FilterEntity::Equals(
                Box::new(FilterEntity::Column("category".to_string())),
                Box::new(FilterEntity::Value(DBValue::String("cat1".to_string()))),
            );
            let result = table.query(query).await;
            assert_eq!(result.len(), 2); // cat1 appears at indices 1 and 4
        }

        let _ = tokio::fs::remove_dir_all("test_db/test_index_persistence").await;
    }

    #[tokio::test]
    async fn test_drop_index() {
        let settings = Settings {
            base_path: "test_db/test_drop_index".to_string(),
        };

        let _ = tokio::fs::remove_dir_all(&settings.base_path).await;

        let mut table = TableRowSchemaless::new("test_table_drop_idx".to_string(), settings).await;
        table.truncate().await;

        // Insert test data
        for i in 0..5 {
            let mut data = HashMap::new();
            data.insert("id".to_string(), DBValue::Number(i as f64));
            data.insert("field".to_string(), DBValue::String(format!("value{}", i)));
            table.insert(data).await;
        }

        // Create index
        table.create_index("field").await;
        assert!(table.list_indexes().contains(&"field".to_string()));

        // Drop index
        table.drop_index("field").await;
        assert!(!table.list_indexes().contains(&"field".to_string()));

        // Query should still work (using full scan)
        let query = FilterEntity::Equals(
            Box::new(FilterEntity::Column("field".to_string())),
            Box::new(FilterEntity::Value(DBValue::String("value2".to_string()))),
        );
        let result = table.query(query).await;
        assert_eq!(result.len(), 1);

        table.drop().await;
        let _ = tokio::fs::remove_dir_all("test_db/test_drop_index").await;
    }

    #[tokio::test]
    async fn test_index_performance() {
        let settings = Settings {
            base_path: "test_db/test_index_performance".to_string(),
        };

        let _ = tokio::fs::remove_dir_all(&settings.base_path).await;

        let mut table = TableRowSchemaless::new("test_table_perf".to_string(), settings).await;
        table.truncate().await;

        // Insert a larger dataset
        for i in 0..1000 {
            let mut data = HashMap::new();
            data.insert("id".to_string(), DBValue::Number(i as f64));
            data.insert(
                "email".to_string(),
                DBValue::String(format!("user{}@example.com", i)),
            );
            table.insert(data).await;
        }

        // Query without index
        let query = FilterEntity::Equals(
            Box::new(FilterEntity::Column("email".to_string())),
            Box::new(FilterEntity::Value(DBValue::String(
                "user500@example.com".to_string(),
            ))),
        );

        let start = std::time::Instant::now();
        let result_no_index = table.query(query.clone()).await;
        let duration_no_index = start.elapsed();

        // Create index
        table.create_index("email").await;

        // Query with index
        let start = std::time::Instant::now();
        let result_with_index = table.query(query).await;
        let duration_with_index = start.elapsed();

        println!("Query without index: {:?}", duration_no_index);
        println!("Query with index: {:?}", duration_with_index);

        // Both should return the same result
        assert_eq!(result_no_index.len(), 1);
        assert_eq!(result_with_index.len(), 1);
        assert_eq!(result_no_index, result_with_index);

        // Index should be faster (though with small dataset difference may be minimal)
        println!(
            "Speedup: {:.2}x",
            duration_no_index.as_secs_f64() / duration_with_index.as_secs_f64()
        );

        table.drop().await;
        let _ = tokio::fs::remove_dir_all("test_db/test_index_performance").await;
    }

    #[tokio::test]
    async fn test_index_with_null_values() {
        let settings = Settings {
            base_path: "test_db/test_index_null".to_string(),
        };

        let _ = tokio::fs::remove_dir_all(&settings.base_path).await;

        let mut table = TableRowSchemaless::new("test_table_null".to_string(), settings).await;
        table.truncate().await;

        // Insert data with null values
        for i in 0..5 {
            let mut data = HashMap::new();
            data.insert("id".to_string(), DBValue::Number(i as f64));
            if i % 2 == 0 {
                data.insert("optional_field".to_string(), DBValue::Null);
            } else {
                data.insert(
                    "optional_field".to_string(),
                    DBValue::String(format!("value{}", i)),
                );
            }
            table.insert(data).await;
        }

        // Create index on field with nulls
        table.create_index("optional_field").await;

        // Query for null values
        let query = FilterEntity::Equals(
            Box::new(FilterEntity::Column("optional_field".to_string())),
            Box::new(FilterEntity::Value(DBValue::Null)),
        );
        let result = table.query(query).await;
        assert_eq!(result.len(), 3); // Indices 0, 2, 4 have null

        table.drop().await;
        let _ = tokio::fs::remove_dir_all("test_db/test_index_null").await;
    }

    #[tokio::test]
    async fn test_multiple_indexes() {
        let settings = Settings {
            base_path: "test_db/test_multiple_indexes".to_string(),
        };

        let _ = tokio::fs::remove_dir_all(&settings.base_path).await;

        let mut table = TableRowSchemaless::new("test_table_multi".to_string(), settings).await;
        table.truncate().await;

        // Insert test data
        for i in 0..10 {
            let mut data = HashMap::new();
            data.insert("id".to_string(), DBValue::Number(i as f64));
            data.insert("name".to_string(), DBValue::String(format!("Person{}", i)));
            data.insert("age".to_string(), DBValue::Number((20 + i) as f64));
            table.insert(data).await;
        }

        // Create multiple indexes
        table.create_index("name").await;
        table.create_index("age").await;

        // Verify both indexes exist
        let indexes = table.list_indexes();
        assert_eq!(indexes.len(), 2);
        assert!(indexes.contains(&"name".to_string()));
        assert!(indexes.contains(&"age".to_string()));

        // Query using first index
        let query1 = FilterEntity::Equals(
            Box::new(FilterEntity::Column("name".to_string())),
            Box::new(FilterEntity::Value(DBValue::String("Person5".to_string()))),
        );
        let result1 = table.query(query1).await;
        assert_eq!(result1.len(), 1);

        // Query using second index
        let query2 = FilterEntity::Equals(
            Box::new(FilterEntity::Column("age".to_string())),
            Box::new(FilterEntity::Value(DBValue::Number(25.0))),
        );
        let result2 = table.query(query2).await;
        assert_eq!(result2.len(), 1);

        table.drop().await;
        let _ = tokio::fs::remove_dir_all("test_db/test_multiple_indexes").await;
    }
}
