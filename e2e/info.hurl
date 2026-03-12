GET {{host}}/info
HTTP 200
[Asserts]
jsonpath "$.name" == "sqlite"
