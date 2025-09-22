SELECT count(*) FROM tenk1 t1
  WHERE t1.thousand = 42 OR t1.thousand = (SELECT t2.tenthous FROM tenk1 t2 WHERE t2.thousand = t1.tenthous + 1 LIMIT 1);
