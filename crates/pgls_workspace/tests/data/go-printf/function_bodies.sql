-- a body with a directive is kept as written, so that the string with it stays the same
create function   f(n int) returns int language sql as $$ select n + %d $$;
-- the printer would requote this body, which would change the string with the directive:
-- the statement stays as written
create function g()   returns text language sql as 'select ''%s''';
