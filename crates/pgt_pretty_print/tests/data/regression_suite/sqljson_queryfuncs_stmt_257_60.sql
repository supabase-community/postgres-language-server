SELECT JSON_VALUE(jsonb '{"d1": "H"}', '$.a2' RETURNING queryfuncs_test_domain DEFAULT 'foo'::queryfuncs_test_domain ON EMPTY);
