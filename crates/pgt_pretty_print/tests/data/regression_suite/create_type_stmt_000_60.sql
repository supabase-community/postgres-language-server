CREATE FUNCTION widget_in(cstring)
   RETURNS widget
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;
