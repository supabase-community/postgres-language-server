select jsonb_insert('{"a": {"b": "value"}}', '{a, c}', '"new_value"', true);
