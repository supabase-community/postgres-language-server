-- A temporary table stays temporary when it is renamed.
create temp table scratch (id int);
alter table scratch rename to scratch_ids;
alter table scratch_ids add column name text;
-- A temporary table created in a transaction that rolled back doesn't exist.
begin;
create temp table rolled_back (id int);
rollback;
-- expect_lint/requireTableSchema
alter table rolled_back add column name text;
