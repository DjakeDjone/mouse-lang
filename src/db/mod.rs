pub mod persistence;

struct DBSettings {
    url: &'static str,
    cache_size: u32,
}

const DB_SETTINGS: DBSettings = DBSettings {
    url: "./mouse-src/data/database.db",
    cache_size: 100,
};
