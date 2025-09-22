SELECT json_agg(q ORDER BY x NULLS FIRST, y)
  FROM rows q;
