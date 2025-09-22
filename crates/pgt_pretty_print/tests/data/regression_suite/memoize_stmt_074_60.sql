INSERT INTO tab_anti SELECT i%3, false FROM generate_series(1,100)i;
