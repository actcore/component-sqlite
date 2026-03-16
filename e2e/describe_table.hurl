# Ensure table exists
POST {{baseurl}}/tools/execute
{
  "arguments": {"sql": "CREATE TABLE IF NOT EXISTS products (id INTEGER PRIMARY KEY, name TEXT NOT NULL, price REAL)"},
  "metadata": {"database_path": "{{db_path}}"}
}
HTTP 200

# Describe it
POST {{baseurl}}/tools/describe-table
{
  "arguments": {"table": "products"},
  "metadata": {"database_path": "{{db_path}}"}
}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "\"name\""
jsonpath "$.content[0].data" contains "\"price\""

# Non-existent table
POST {{baseurl}}/tools/describe-table
{
  "arguments": {"table": "nonexistent"},
  "metadata": {"database_path": "{{db_path}}"}
}
HTTP *
[Asserts]
jsonpath "$.error.kind" == "std:not-found"
