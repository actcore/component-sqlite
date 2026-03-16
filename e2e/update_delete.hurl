# Setup table
POST {{baseurl}}/tools/execute
{"arguments": {"sql": "CREATE TABLE IF NOT EXISTS items (id INTEGER PRIMARY KEY, name TEXT, qty INTEGER)"}, "metadata": {"database_path": "{{db_path}}"}}
HTTP 200

# Insert rows
POST {{baseurl}}/tools/execute
{"arguments": {"sql": "INSERT INTO items (name, qty) VALUES ('apple', 10), ('banana', 5), ('cherry', 3)"}, "metadata": {"database_path": "{{db_path}}"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "\"rows_affected\":3"

# Update
POST {{baseurl}}/tools/execute
{"arguments": {"sql": "UPDATE items SET qty = 20 WHERE name = 'apple'"}, "metadata": {"database_path": "{{db_path}}"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "\"rows_affected\":1"

# Verify update
POST {{baseurl}}/tools/query
{"arguments": {"sql": "SELECT qty FROM items WHERE name = 'apple'"}, "metadata": {"database_path": "{{db_path}}"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "20"

# Delete
POST {{baseurl}}/tools/execute
{"arguments": {"sql": "DELETE FROM items WHERE qty < 10"}, "metadata": {"database_path": "{{db_path}}"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "\"rows_affected\":2"

# Verify remaining
POST {{baseurl}}/tools/query
{"arguments": {"sql": "SELECT COUNT(*) as cnt FROM items"}, "metadata": {"database_path": "{{db_path}}"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "cnt"
