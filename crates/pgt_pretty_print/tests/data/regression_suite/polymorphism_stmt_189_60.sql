select first_el_agg_f8(x::float8) over(order by x) from generate_series(1,10) x;
