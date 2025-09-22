SELECT jsa FROM jsonb_populate_record(NULL::jsbrec, '{"jsa": 123}') q;
