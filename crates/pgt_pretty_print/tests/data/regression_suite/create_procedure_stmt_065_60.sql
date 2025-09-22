CREATE PROCEDURE ptest10(IN a int, IN b int, IN c int)
LANGUAGE SQL AS $$ SELECT a + b - c $$;
