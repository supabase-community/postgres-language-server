SELECT count(*) FROM tenk1 LEFT JOIN tenk2 ON
  tenk1.hundred = 42 AND (tenk2.thousand = 42 OR tenk2.thousand = 41 OR tenk2.tenthous = 2) AND
  tenk2.hundred = tenk1.hundred;
