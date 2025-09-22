CREATE INDEX ON test_jsonb_mutability (JSON_QUERY(js, '$.a ? (@.date() < $.datetime("HH:MI TZH"))'));
