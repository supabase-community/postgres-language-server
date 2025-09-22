SELECT reca FROM jsonb_populate_record(NULL::jsbrec, '{"reca": [{"a": "abc", "b": 456}, null, {"c": "01.02.2003", "x": 43.2}]}') q;
