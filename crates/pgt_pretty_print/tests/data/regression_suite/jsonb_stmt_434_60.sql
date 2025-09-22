SELECT rec FROM jsonb_populate_record(NULL::jsbrec, '{"rec": {"a": "abc", "c": "01.02.2003", "x": 43.2}}') q;
