# Create table
POST {{host}}/tools/execute
{
  "arguments": {"sql": "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER)"},
  "config": {"database_path": "{{db_path}}"}
}
HTTP 200

# Insert row
POST {{host}}/tools/execute
{
  "arguments": {"sql": "INSERT INTO users (name, age) VALUES (?1, ?2)", "params": ["Alice", 30]},
  "config": {"database_path": "{{db_path}}"}
}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "\"rows_affected\":1"

# Query
POST {{host}}/tools/query
{
  "arguments": {"sql": "SELECT * FROM users"},
  "config": {"database_path": "{{db_path}}"}
}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "Alice"
jsonpath "$.content[0].data" contains "30"
