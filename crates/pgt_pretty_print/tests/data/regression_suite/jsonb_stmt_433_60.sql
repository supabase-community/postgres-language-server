SELECT rec FROM jsonb_populate_record(NULL::jsbrec, '{"rec": [1, 2]}') q;
