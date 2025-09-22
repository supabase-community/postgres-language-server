CREATE FUNCTION test_canonicalize_path(text)
   RETURNS text
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;
