CREATE INDEX ON test_jsonb_mutability (JSON_VALUE(js, '$' DEFAULT random()::int ON ERROR));
