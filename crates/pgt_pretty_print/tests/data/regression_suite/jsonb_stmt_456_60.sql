select * from jsonb_populate_record(NULL::jsb_ia2, '{"a": [[1, 0], [2, 3]]}') q;
