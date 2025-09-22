select a, b, count(*) from gstest2 group by grouping sets ((a, b), (a)) having a > 1 and b > 1;
