SELECT count(*) FROM prt1 t1 LEFT JOIN LATERAL
			  (SELECT t1.b AS t1b, t2.* FROM prt2 t2) s
			  ON t1.a = s.b WHERE s.t1b = s.a;
