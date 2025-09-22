select * from json_to_record('{"ia": [[1], 2]}') as x(ia _int4);
