create function infinite_recurse() returns int as
'select infinite_recurse()' language sql;
