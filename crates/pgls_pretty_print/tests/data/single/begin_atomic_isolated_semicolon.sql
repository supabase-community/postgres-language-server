-- pgls-format: isolateSemicolon=true, layout=fit
CREATE FUNCTION f() RETURNS int LANGUAGE SQL BEGIN ATOMIC SELECT 1; SELECT a, b, c FROM a_rather_long_table_name_to_break_the_line_somewhere_here WHERE a = 1 AND b = 2 AND c = 3; END;
