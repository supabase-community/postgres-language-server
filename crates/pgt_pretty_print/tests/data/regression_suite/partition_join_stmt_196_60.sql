SELECT t1.a, t1.c, t2.b, t2.c FROM prt1_l t1, prt2_l t2 WHERE t1.a = t2.a AND t1.a = t2.b AND t1.c = t2.c ORDER BY t1.a, t2.b;
