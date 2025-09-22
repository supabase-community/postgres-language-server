select jsonb_insert('{"a": []}', '{a, 1}', '"new_value"', true);
