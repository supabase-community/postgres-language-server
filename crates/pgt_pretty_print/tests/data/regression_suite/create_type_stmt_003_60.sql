CREATE FUNCTION int44out(city_budget)
   RETURNS cstring
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;
