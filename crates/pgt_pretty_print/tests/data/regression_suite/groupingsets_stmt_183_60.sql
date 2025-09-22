select a < b and b < 3 from (values (1, 2)) t(a, b) group by rollup(a < b and b < 3) having a < b and b < 3;
