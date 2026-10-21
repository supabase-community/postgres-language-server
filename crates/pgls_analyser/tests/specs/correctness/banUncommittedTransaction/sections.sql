-- A section that the migration tool runs in a transaction is committed by the tool.
-- section: up
begin;
create table users (id bigint);

-- section: down transaction:false
-- expect_lint/banUncommittedTransaction
begin;
drop table users;
