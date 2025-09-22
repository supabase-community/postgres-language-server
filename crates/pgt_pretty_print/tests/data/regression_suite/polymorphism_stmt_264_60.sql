create function dfunc(a variadic int[]) returns int as
$$ select array_upper($1, 1) $$ language sql;
