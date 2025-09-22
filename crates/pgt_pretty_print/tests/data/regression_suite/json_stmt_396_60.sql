select json_object_agg_unique(mod(i,100), i) from generate_series(0, 199) i;
