CREATE INDEX ON test_jsonb_mutability (JSON_QUERY(js, '$.a ? (@.time() < $.time_tz())'));
