-- expect_lint/banReplaceViewFunction
create or replace procedure p() language sql as $$ select 1 $$;
