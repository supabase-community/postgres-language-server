select a,count(*) from gstest2 group by rollup(a) order by a;
