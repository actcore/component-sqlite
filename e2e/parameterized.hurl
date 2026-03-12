# Setup
POST {{host}}/tools/execute
{"arguments": {"sql": "CREATE TABLE IF NOT EXISTS params_test (id INTEGER PRIMARY KEY, val TEXT)"}, "config": {"database_path": "{{db_path}}"}}
HTTP 200

# Parameterized insert
POST {{host}}/tools/execute
{"arguments": {"sql": "INSERT INTO params_test (val) VALUES (?1)", "params": ["hello"]}, "config": {"database_path": "{{db_path}}"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "\"rows_affected\":1"

# Parameterized query
POST {{host}}/tools/query
{"arguments": {"sql": "SELECT val FROM params_test WHERE val = ?1", "params": ["hello"]}, "config": {"database_path": "{{db_path}}"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "hello"

# Multiple params
POST {{host}}/tools/execute
{"arguments": {"sql": "INSERT INTO params_test (val) VALUES (?1), (?2)", "params": ["foo", "bar"]}, "config": {"database_path": "{{db_path}}"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "\"rows_affected\":2"
