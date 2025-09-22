insert into t select mod(i,10),mod(i,10),i from generate_series(1,10000) s(i);
