SELECT (test_json->>3) IS NULL AS expect_true FROM test_jsonb WHERE json_type = 'array';
