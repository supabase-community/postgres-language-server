DO $$
DECLARE
  o int;
  a int[] := ARRAY[1,2,3,2,3,1,2];
BEGIN
  o := array_position(a, 2);
  WHILE o IS NOT NULL
  LOOP
    RAISE NOTICE '%', o;
    o := array_position(a, 2, o + 1);
  END LOOP;
END
$$ LANGUAGE plpgsql;
