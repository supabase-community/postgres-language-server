select jsonb_insert('{"a": [0,1,2]}', '{a, 2}', '"new_value"', true);
