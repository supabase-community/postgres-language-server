select rank('3') within group (order by x) from generate_series(1,5) x;
