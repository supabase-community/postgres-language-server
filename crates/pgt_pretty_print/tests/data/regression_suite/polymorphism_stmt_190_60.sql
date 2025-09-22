select first_el_agg_any(x) over(order by x) from generate_series(1,10) x;
