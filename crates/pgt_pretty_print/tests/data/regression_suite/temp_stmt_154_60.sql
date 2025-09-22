CREATE FUNCTION test_temp_pin(p_start int, p_end int)
RETURNS void
LANGUAGE plpgsql
AS $f$
  DECLARE
      cursorname text;
      query text;
  BEGIN
    FOR i IN p_start..p_end LOOP
       cursorname = 'c_'||i;
       query = format($q$DECLARE %I CURSOR FOR SELECT ctid FROM test_temp WHERE ctid >= '( %s, 1)'::tid $q$, cursorname, i);
       EXECUTE query;
       EXECUTE 'FETCH NEXT FROM '||cursorname;
       -- for test development
       -- RAISE NOTICE '%: %', cursorname, query;
    END LOOP;
  END;
$f$;
