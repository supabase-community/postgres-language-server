select jsonb_path_query('[12, {"a": 13}, {"b": 14}]', 'lax $[1].a');
