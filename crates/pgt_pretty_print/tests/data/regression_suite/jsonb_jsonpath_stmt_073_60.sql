select jsonb_path_query('[12, {"a": 13}, {"b": 14}]', 'lax $[0 to 10].a');
