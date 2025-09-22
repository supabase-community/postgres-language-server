CREATE INDEX ON test_jsonb_mutability (JSON_QUERY(js, '$.a ? (@.timestamp_tz() < $.timestamp_tz())'));
