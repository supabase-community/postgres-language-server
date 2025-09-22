CREATE FUNCTION foolme(timestamptz DEFAULT clock_timestamp())
  RETURNS timestamptz
  IMMUTABLE AS 'select $1' LANGUAGE sql;
