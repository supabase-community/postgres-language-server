SELECT t1.a, t1.c, t2.b, t2.c FROM prt1 t1, prt2 t2 WHERE t1.a = t2.a AND t1.a = t2.b ORDER BY t1.a, t2.b;
