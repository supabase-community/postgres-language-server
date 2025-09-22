SELECT row_to_json(row((select array_agg(x) as d from generate_series(5,10) x)),false);
