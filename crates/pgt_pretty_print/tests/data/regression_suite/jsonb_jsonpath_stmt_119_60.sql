select jsonb_path_query('{"a": {"b": 1}}', 'lax $.**.b ? (@ > 0)');
