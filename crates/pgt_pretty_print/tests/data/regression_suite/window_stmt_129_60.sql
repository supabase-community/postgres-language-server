CREATE FUNCTION unbounded_syntax_test2b(unbounded int) RETURNS TABLE (a int, b int, c int)
LANGUAGE SQL
AS $$
  SELECT sum(unique1) over (rows between unbounded preceding and unbounded following),
         unique1, four
  FROM tenk1 WHERE unique1 < 10;
$$;
