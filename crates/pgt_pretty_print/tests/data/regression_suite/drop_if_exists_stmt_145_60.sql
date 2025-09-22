CREATE FUNCTION test_ambiguous_funcname(text) returns text as $$ select $1; $$ language sql;
