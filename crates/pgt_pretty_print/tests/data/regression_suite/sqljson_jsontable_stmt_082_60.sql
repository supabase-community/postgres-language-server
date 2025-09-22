SELECT * FROM JSON_TABLE('{"a": [{"b": "1"}, {"b": "2"}]}', '$' COLUMNS (b json path '$.a[*].b' ERROR ON ERROR));
