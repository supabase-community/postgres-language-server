CREATE FUNCTION widget_out(widget)
   RETURNS cstring
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;
