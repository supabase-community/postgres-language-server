SELECT row_to_json(j)::jsonb FROM (
  SELECT left(E'abcdefghijklmnopqrstuv"\twxyz012345678', a) AS a
  FROM generate_series(0,37) a
) j;
