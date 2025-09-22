SELECT JSON_QUERY(js, '$'  RETURNING int DEFAULT ret_setint() ON ERROR) FROM test_jsonb_mutability;
