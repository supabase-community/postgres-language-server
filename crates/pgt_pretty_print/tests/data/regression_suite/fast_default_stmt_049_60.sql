CREATE OR REPLACE FUNCTION foo(a INT) RETURNS TEXT AS $$
DECLARE res TEXT := '';
        i INT;
BEGIN
  i := 0;
  WHILE (i < a) LOOP
    res := res || chr(ascii('a') + i);
    i := i + 1;
  END LOOP;
  RETURN res;
END; $$ LANGUAGE PLPGSQL STABLE;
