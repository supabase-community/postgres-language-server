CREATE FUNCTION test_nested() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  EXECUTE $inner$SELECT 1$inner$;
END;
$$;
