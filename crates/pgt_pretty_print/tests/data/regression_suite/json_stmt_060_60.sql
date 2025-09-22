SELECT array_to_json(array_agg(x),false) from generate_series(5,10) x;
