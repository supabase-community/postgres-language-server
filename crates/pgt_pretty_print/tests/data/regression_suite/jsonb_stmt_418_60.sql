SELECT js FROM jsonb_populate_record(NULL::jsbrec, '{"js": "abc"}') q;
