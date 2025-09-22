select jsonb_path_query('{"a": [1,2,3], "b": [3,4,5]}', '$.*');
