DO $$
DECLARE
  i int;
BEGIN
  FOR i IN 1_001..1_003 LOOP
    RAISE NOTICE 'i = %', i;
  END LOOP;
END $$;
