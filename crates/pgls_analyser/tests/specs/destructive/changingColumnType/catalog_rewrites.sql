-- With the old type of the column known, a change that isn't binary coercible rewrites the table,
-- even to text, and so does a USING clause. The table is created in a transaction of its own,
-- since a table created in the same transaction isn't reported.
-- catalog: builtin
begin;
create table recipes (id int, count int, body text, size int);
commit;
begin;
-- expect_lint/changingColumnType
alter table recipes alter column id type text;
-- expect_lint/changingColumnType
alter table recipes alter column count type varchar;
-- expect_lint/changingColumnType
alter table recipes alter column body type text using body || '';
-- expect_lint/changingColumnType
alter table recipes alter column size type int[] using array[size];
commit;
