CREATE FUNCTION unbounded_syntax_test1b(x int) RETURNS TABLE (a int, b int, c int)
LANGUAGE SQL
AS $$
  SELECT sum(unique1) over (rows between x preceding and x following),
         unique1, four
  FROM tenk1 WHERE unique1 < 10;
$$;
