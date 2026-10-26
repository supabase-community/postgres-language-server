create temp table scratch (id int);
drop table scratch;
-- After the drop, the name refers to a permanent table again.
-- expect_lint/requireTableSchema
alter table scratch add column name text;
