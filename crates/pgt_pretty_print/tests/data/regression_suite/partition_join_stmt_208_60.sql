SELECT COUNT(*) FROM prt1_l t1 LEFT JOIN LATERAL
			  (SELECT t1.b AS t1b, t2.* FROM prt2_l t2) s
			  ON t1.a = s.b AND t1.b = s.a AND t1.c = s.c
			  WHERE s.t1b = s.a;
