CREATE STATISTICS ab1_exprstat_6 ON
  (case a when 1 then true else false end), b FROM ab1;
