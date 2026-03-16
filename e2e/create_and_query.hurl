# Create table
POST {{baseurl}}/tools/execute
{
  "arguments": {"sql": "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER)"},
  "metadata": {"database_path": "{{db_path}}"}
}
HTTP 200

# Insert row
POST {{baseurl}}/tools/execute
{
  "arguments": {"sql": "INSERT INTO users (name, age) VALUES (?1, ?2)", "params": ["Alice", 30]},
  "metadata": {"database_path": "{{db_path}}"}
}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "\"rows_affected\":1"

# Query
POST {{baseurl}}/tools/query
{
  "arguments": {"sql": "SELECT * FROM users"},
  "metadata": {"database_path": "{{db_path}}"}
}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "Alice"
jsonpath "$.content[0].data" contains "30"
