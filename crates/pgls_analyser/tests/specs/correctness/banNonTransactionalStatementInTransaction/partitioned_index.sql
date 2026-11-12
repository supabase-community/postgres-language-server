create table events (id bigint, created_at timestamptz) partition by range (created_at);
create index events_created_at_idx on events (created_at);
create table users (id bigint);
create index users_id_idx on users (id);
create schema app;
create table app.logs (id bigint, logged_at timestamptz) partition by range (logged_at);
create index logs_logged_at_idx on app.logs (logged_at);
begin;
-- The index of a partitioned table is a partitioned index, which Postgres reindexes partition
-- by partition, each in a transaction of its own.
-- expect_lint/banNonTransactionalStatementInTransaction
reindex index events_created_at_idx;
-- expect_lint/banNonTransactionalStatementInTransaction
reindex index app.logs_logged_at_idx;
alter index events_created_at_idx rename to events_idx;
-- expect_lint/banNonTransactionalStatementInTransaction
reindex index events_idx;
reindex index users_id_idx;
-- The index is in `app`, which isn't on the search path.
reindex index logs_logged_at_idx;
commit;
