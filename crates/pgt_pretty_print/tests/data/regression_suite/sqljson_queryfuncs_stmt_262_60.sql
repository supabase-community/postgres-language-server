SELECT JSON_QUERY('"a"', '$.a' RETURNING queryfuncs_test_domain DEFAULT (select '"1"')::queryfuncs_test_domain ON ERROR);
