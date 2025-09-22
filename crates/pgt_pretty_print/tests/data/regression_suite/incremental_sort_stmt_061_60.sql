insert into t(a, b) select i, i from generate_series(1, 1000) n(i);
