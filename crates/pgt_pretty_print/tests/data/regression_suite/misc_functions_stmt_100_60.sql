SELECT * FROM tenk1 a JOIN my_gen_series(1,1000) g ON a.unique1 = g;
