SELECT JSON_QUERY(js, '$'  RETURNING int DEFAULT sum(1) over() ON ERROR) FROM test_jsonb_mutability;
