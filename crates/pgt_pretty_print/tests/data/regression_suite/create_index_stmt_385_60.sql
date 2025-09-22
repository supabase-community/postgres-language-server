SELECT count(*) FROM tenk1, tenk2
  WHERE tenk1.hundred = 42 AND (tenk2.thousand = 42 OR tenk1.thousand = 41 OR tenk2.tenthous = 2) AND
  tenk2.hundred = tenk1.hundred;
