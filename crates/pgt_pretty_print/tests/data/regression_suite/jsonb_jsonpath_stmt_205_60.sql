select jsonb_path_query('[1,2,3,{"b": [3,4,5]}]', 'strict $.*');
