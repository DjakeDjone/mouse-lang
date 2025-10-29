# MouseDB 🐁

MouseDB is a simple database in mouse-lang, written in Rust.

## Example Usage

```mouse

db.crateTable("users"); // Create a table called "users", pk is "user_id" per default (auto-incremented)

db.insert("users", {
    "name": "John Doe",
    "email": "john@example.com"
});

db.selectOne("users", "user_id = 1")


db.createTSTable("logs"); // Time series table automatically has a timestamp column that is automatically generated if not provided

db.insert("logs", {
    "message": "User logged in",
    "level": 1,
    "user_id": 1,
    "timestamp": "2022-01-01T00:00:00Z"
});

db.insert("logs", {
    "message": "System crashed",
    "level": 2,
});

db.insert("logs", {
    "message": "User logged out",
    "user_id": 1,
    "level": 1,
});

db.avg("logs", "level", "timestamp > '2022-01-01T00:00:00Z'");



```
