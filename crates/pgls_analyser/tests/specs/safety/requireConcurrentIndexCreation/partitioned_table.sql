-- section: up
create table events (id bigint, created_at timestamptz) partition by range (created_at);

-- Postgres can't create an index on a partitioned table concurrently, so there is no fix.
-- section: down transaction:false
-- expect_lint/requireConcurrentIndexCreation
create index events_created_at_idx on events (created_at);
