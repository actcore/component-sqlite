POST {{host}}/tools
HTTP 200
[Asserts]
jsonpath "$.tools[*].name" includes "query"
jsonpath "$.tools[*].name" includes "execute"
jsonpath "$.tools[*].name" includes "list-tables"
jsonpath "$.tools[*].name" includes "describe-table"
jsonpath "$.tools[*].name" includes "execute-batch"
