SELECT * FROM JSON_TABLE(jsonb '{"d1": "H"}', '$'
    COLUMNS (js1 jsonb_test_domain PATH '$.a2' DEFAULT 'foo'::jsonb_test_domain ON EMPTY));
