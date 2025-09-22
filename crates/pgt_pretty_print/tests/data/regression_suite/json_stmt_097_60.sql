SELECT test_json->'field2'
FROM test_json
WHERE json_type = 'object';
