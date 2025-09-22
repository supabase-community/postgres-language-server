select jsonb_insert('{"a": {"b": "value"}}', '{a, b}', '"new_value"');
