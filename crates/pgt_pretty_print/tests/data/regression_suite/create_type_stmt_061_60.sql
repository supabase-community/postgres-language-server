CREATE FUNCTION pt_in_widget(point, widget)
   RETURNS bool
   AS 'regresslib'
   LANGUAGE C STRICT;
