SELECT *
FROM
	(VALUES ('1'), ('"err"')) vals(js),
	JSON_TABLE(vals.js::jsonb, '$' COLUMNS (a int PATH '$')) jt;
