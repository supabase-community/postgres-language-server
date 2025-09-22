SELECT js FROM jsonb_populate_record(NULL::jsbrec, '{"js": 123.45}') q;
