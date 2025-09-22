select jsonb_insert('{"a": [0,1,2]}', '{a, 1}', '"new_value"');
