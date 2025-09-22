CREATE INDEX ON test_jsonb_mutability (JSON_QUERY(js, '$.date() < $x' PASSING '12:34'::timetz AS x));
