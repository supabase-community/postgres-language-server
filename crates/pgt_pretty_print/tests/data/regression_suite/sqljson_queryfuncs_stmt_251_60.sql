SELECT JSON_QUERY(js, '$'  RETURNING int DEFAULT b + 1 ON ERROR) FROM test_jsonb_mutability;
