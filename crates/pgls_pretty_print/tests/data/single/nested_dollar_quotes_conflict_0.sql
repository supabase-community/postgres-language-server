CREATE FUNCTION test_conflict() RETURNS void LANGUAGE plpgsql AS $outer$
BEGIN
  EXECUTE $$SELECT 1$$;
END;
$outer$;
