-- An unqualified name refers to the temporary table created earlier in the file.
-- expect_no_diagnostics
create temp table scratch (id int);
alter table scratch add column name text;
create index on scratch (id);
drop table scratch;
create table pg_temp.other_scratch (id int);
alter table other_scratch add column name text;
select 1 as id into temp scratch_ids;
drop table scratch_ids;
-- Queries, like the one of a view, are not checked.
create view public.v as select * from users;
