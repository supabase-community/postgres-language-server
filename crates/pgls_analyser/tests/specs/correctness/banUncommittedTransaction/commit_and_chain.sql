begin;
create table users (id bigint);
-- expect_lint/banUncommittedTransaction
commit and chain;
create table posts (id bigint);
