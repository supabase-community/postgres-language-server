CREATE TABLE test_jsonb_constraints (
	js text,
	i int,
	x jsonb DEFAULT JSON_QUERY(jsonb '[1,2]', '$[*]' WITH WRAPPER)
	CONSTRAINT test_jsonb_constraint1
		CHECK (js IS JSON)
	CONSTRAINT test_jsonb_constraint2
		CHECK (JSON_EXISTS(js::jsonb, '$.a' PASSING i + 5 AS int, i::text AS "TXT", array[1,2,3] as arr))
	CONSTRAINT test_jsonb_constraint3
		CHECK (JSON_VALUE(js::jsonb, '$.a' RETURNING int DEFAULT '12' ON EMPTY ERROR ON ERROR) > i)
	CONSTRAINT test_jsonb_constraint4
		CHECK (JSON_QUERY(js::jsonb, '$.a' WITH CONDITIONAL WRAPPER EMPTY OBJECT ON ERROR) = jsonb '[10]')
	CONSTRAINT test_jsonb_constraint5
		CHECK (JSON_QUERY(js::jsonb, '$.a' RETURNING char(5) OMIT QUOTES EMPTY ARRAY ON EMPTY) >  'a' COLLATE "C")
);
