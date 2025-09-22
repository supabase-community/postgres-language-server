SELECT array_agg(i) OVER w
  FROM generate_series(1,5) i
WINDOW w AS (ORDER BY i ROWS BETWEEN (('foo' < 'foobar')::integer) PRECEDING AND CURRENT ROW);
