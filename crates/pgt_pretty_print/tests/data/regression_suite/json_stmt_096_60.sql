SELECT test_json -> 'x'
FROM test_json
WHERE json_type = 'object';
