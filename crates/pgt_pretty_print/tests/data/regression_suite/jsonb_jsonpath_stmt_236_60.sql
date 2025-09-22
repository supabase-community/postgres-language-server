select jsonb_path_query('{"a": 2}', '($.a - 5).abs() + 10');
