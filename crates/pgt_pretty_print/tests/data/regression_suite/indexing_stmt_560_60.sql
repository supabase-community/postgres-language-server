create or replace function test_pg_index_toast_func (a int, b int[])
  returns bool as $$ select true $$ language sql immutable;
