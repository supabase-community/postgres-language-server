create or replace function rngfuncbar(out integer, out numeric) as
$$ select (1, 2.1) $$ language sql;
