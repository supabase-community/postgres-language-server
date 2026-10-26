begin;
create table users (id bigint);
commit;
-- expect_lint/banUncommittedTransaction
start transaction;
create table posts (id bigint);
