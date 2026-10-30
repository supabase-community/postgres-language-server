-- The file ends without a semicolon, so adding `commit;` after the last statement would make it
-- `create table ... commit;`. The fix is left out.
-- expect_lint/banUncommittedTransaction
begin;
create table users (id bigint)