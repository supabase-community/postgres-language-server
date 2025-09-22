select row_to_json(s.*) from generate_series(11,14) with ordinality s;
