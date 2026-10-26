create table events (id bigint, created_at timestamptz) partition by range (created_at);
create index events_created_at_idx on events (created_at);
create table users (id bigint);
create index users_id_idx on users (id);
begin;
-- Postgres processes the partitions one by one, each in a transaction of its own.
-- expect_lint/banNonTransactionalStatementInTransaction
reindex table events;
-- expect_lint/banNonTransactionalStatementInTransaction
cluster events using events_created_at_idx;
reindex table users;
cluster users using users_id_idx;
commit;
