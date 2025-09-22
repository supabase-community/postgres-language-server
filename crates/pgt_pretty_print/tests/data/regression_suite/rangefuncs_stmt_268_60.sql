CREATE FUNCTION bad (f1 anyarray, out f2 anycompatible, out f3 anycompatiblearray)
AS 'select $1, array[$1,$1]' LANGUAGE sql;
