select a, b, sum(v), count(*) from gstest_empty group by grouping sets ((a,b),a);
