SELECT * FROM tenk1 a JOIN my_gen_series(1,10) g ON a.unique1 = g;
