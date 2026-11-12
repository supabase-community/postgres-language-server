-- The COMMIT goes after the function, not into its body.
-- expect_lint/banUncommittedTransaction
begin;
create function one() returns int language sql as $$ select 1 $$;
