insert into t(a, b) select i / 10, i from generate_series(1, 1000) n(i);
