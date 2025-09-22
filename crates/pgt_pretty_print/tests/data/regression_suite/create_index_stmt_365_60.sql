SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1::int2 OR tenthous::int2 = 3::int8 OR tenthous::int2 = 42::int8);
