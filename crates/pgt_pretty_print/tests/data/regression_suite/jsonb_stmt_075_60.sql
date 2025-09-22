SELECT jsonb_agg(q ORDER BY x, y)
  FROM rows q;
