select sum(v), count(*) from gstest_empty group by grouping sets ((),(),());
