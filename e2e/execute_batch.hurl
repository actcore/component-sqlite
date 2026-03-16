POST {{baseurl}}/tools/execute-batch
{
  "arguments": {"sql": "CREATE TABLE IF NOT EXISTS batch_a (x INT); CREATE TABLE IF NOT EXISTS batch_b (y INT);"},
  "metadata": {"database_path": "{{db_path}}"}
}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "ok"
