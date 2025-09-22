select not a from (values(true)) t(a) group by rollup(not a) having not not a;
