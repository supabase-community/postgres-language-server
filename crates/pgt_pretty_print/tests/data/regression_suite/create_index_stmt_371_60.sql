SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1::numeric OR tenthous = 3::int4 OR tenthous = 42::numeric);
