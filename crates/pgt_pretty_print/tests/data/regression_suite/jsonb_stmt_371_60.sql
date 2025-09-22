SELECT i FROM jsonb_populate_record(NULL::jsbrec_i_not_null, '{"i": null}') q;
