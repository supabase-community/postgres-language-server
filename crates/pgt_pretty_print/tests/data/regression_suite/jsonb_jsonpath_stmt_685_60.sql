select jsonb_path_query('"2017-03-10 12:34:56+3:10"', '$.datetime().type()');
