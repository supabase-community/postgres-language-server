CREATE PROCEDURE ptest10(OUT a int, IN b int, IN c int)
LANGUAGE SQL AS $$ SELECT b - c $$;
