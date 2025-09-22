SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1::int2 OR tenthous = 3::int8 OR tenthous = 42::int8);
