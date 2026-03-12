# Ensure table exists first
POST {{host}}/tools/execute
{
  "arguments": {"sql": "CREATE TABLE IF NOT EXISTS test_list (id INTEGER PRIMARY KEY)"},
  "config": {"database_path": "{{db_path}}"}
}
HTTP 200

POST {{host}}/tools/list-tables
{
  "arguments": {},
  "config": {"database_path": "{{db_path}}"}
}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "test_list"
