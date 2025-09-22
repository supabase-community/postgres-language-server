CREATE FUNCTION test_ambiguous_funcname(int) returns int as $$ select $1; $$ language sql;
