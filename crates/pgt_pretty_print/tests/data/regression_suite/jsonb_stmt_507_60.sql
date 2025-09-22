select * from jsonb_to_record('{"ia": [1, "2", null, 4]}') as x(ia _int4);
