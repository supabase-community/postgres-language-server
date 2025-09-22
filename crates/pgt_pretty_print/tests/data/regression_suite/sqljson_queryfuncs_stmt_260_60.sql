SELECT JSON_VALUE(jsonb '{"d1": "foo"}', '$.a2' RETURNING queryfuncs_test_domain DEFAULT 'foo1'::queryfuncs_test_domain ON EMPTY);
