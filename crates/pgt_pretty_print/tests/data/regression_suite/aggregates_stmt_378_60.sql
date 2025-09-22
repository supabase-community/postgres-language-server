select percent_rank(0) within group (order by x) from generate_series(1,0) x;
