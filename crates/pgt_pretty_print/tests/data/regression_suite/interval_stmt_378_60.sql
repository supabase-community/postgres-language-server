CREATE FUNCTION eval(expr text)
RETURNS text AS
$$
DECLARE
  result text;
BEGIN
  EXECUTE 'select '||expr INTO result;
  RETURN result;
EXCEPTION WHEN OTHERS THEN
  RETURN SQLERRM;
END
$$
LANGUAGE plpgsql;
