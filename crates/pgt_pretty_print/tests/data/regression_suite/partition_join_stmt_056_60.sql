SELECT * FROM prt1 t1 JOIN prt1 t2 ON t1.a = t2.a WHERE t1.a IN (SELECT a FROM prt1 t3);
