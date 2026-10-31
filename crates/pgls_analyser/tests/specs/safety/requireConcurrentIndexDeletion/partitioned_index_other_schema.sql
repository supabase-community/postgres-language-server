-- An index of a partitioned table in another schema doesn't keep the fix from the index on the
-- search path.
-- section: up transaction:false queries:separate
create table events (id bigint, created_at timestamptz);
create index events_created_at_idx on events (created_at);
create table archive.events (id bigint, created_at timestamptz) partition by range (created_at);
create index events_created_at_idx on archive.events (created_at);
-- expect_lint/requireConcurrentIndexDeletion
drop index events_created_at_idx;
