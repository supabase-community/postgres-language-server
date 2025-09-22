SELECT JSON_QUERY(js, '$'  RETURNING int DEFAULT (SELECT 1) ON ERROR) FROM test_jsonb_mutability;
