select jsonb_path_query('{"a": 2.5}', '-($.a * $.a).floor() % 4.3');
