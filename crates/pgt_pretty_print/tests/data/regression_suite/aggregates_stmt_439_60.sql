SELECT y, x, array_agg(distinct w)
  FROM btg WHERE y < 0 GROUP BY x, y;
