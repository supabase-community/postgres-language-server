SELECT * FROM JSON_TABLE(jsonb '{"d1": "foo"}', '$'
    COLUMNS (js1 jsonb_test_domain PATH '$.d1' DEFAULT 'foo2'::jsonb_test_domain ON ERROR));
