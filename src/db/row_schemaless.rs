use chumsky::prelude::todo;

use crate::db::query_engine::PreSelectedField;
use crate::db::{query_engine, DBValue, DBValueType, FilterEntity};
use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::iter::Map;

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
            std::fs::create_dir_all(&settings.base_path).expect("Failed to create directory");
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
            .unwrap();

        writeln!(file, "{}", serde_json::to_string(&data).unwrap()).unwrap();
    }

    pub async fn drop(&mut self) {
        // TODO: delete cache
        // delete the file
        std::fs::remove_file(format!("{}/{}", self.settings.base_path, self.primary_key))
            .expect("Failed to remove file");
    }

    pub async fn truncate(&mut self) {
        // TODO: delete cache
        // remove all rows
        std::fs::remove_file(format!("{}/{}", self.settings.base_path, self.primary_key))
            .expect("Failed to remove file");
        std::fs::File::create(format!("{}/{}", self.settings.base_path, self.primary_key))
            .expect("Failed to create file");
    }

    pub async fn query(&self, query: FilterEntity) -> Vec<HashMap<String, DBValue>> {
        let necessary_fields = query_engine::pre_select(&query).unwrap_or(
            self.known_columns
                .iter()
                .map(|col| PreSelectedField::from_column(col.clone()))
                .collect(),
        );

        // later do preselect with partitioning etc.
        // for now just go though every row
        let mut result = Vec::new();
        use std::io::BufRead;
        use std::io::BufReader;
        let file = OpenOptions::new()
            .read(true)
            .open(format!("{}/{}", self.settings.base_path, self.primary_key))
            .unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let row: HashMap<String, DBValue> = serde_json::from_str(&line).unwrap();
            if query_engine::execute_query(&query, &row) {
                result.push(row);
            }
        }
        result
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

        let result = rows[0].clone();
        table.drop().await;

        // assert_eq!(result.get("id").unwrap(), &DBValue::Number(1.0));
    }
}
