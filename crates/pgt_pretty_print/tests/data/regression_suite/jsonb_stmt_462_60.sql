select * from jsonb_populate_record(NULL::jsb_i_not_null_rec, '{"a": null}') q;
