CREATE FUNCTION gtest_trigger_func4() RETURNS trigger
  LANGUAGE plpgsql
AS $$
BEGIN
  NEW.a = 10;
  NEW.b = 300;
  RETURN NEW;
END;
$$;
