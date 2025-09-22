CREATE FUNCTION leak(integer,integer) RETURNS boolean
  AS 'int4lt'
  LANGUAGE internal IMMUTABLE STRICT;
