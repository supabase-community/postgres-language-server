-- The file ends without a semicolon, so the fix adds one before `commit;`.
-- expect_lint/banUncommittedTransaction
begin;
create table users (id bigint)