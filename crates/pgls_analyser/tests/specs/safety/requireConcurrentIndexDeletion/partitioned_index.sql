-- Postgres can't drop the index of a partitioned table concurrently, so there is no fix.
-- section: up transaction:false queries:separate
create table events (id bigint, created_at timestamptz) partition by range (created_at);
create index events_created_at_idx on events (created_at);
-- expect_lint/requireConcurrentIndexDeletion
drop index events_created_at_idx;
create table users (id bigint, email text);
create index users_email_idx on users (email);
-- expect_lint/requireConcurrentIndexDeletion
drop index users_email_idx;
