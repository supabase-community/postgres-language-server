-- expect_lint/banReplaceViewFunction
create or replace function f() returns int language sql as $$ select 1 $$;
