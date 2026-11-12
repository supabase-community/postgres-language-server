-- Without the old type of the column, only the changes to text, varchar and numeric without a
-- modifier count as safe, unless there is a USING clause.
-- catalog: builtin
-- expect_lint/changingColumnType
alter table unknown_table alter column id type bigint;
-- expect_lint/changingColumnType
alter table unknown_table alter column id type text using id::text;
alter table unknown_table alter column id type text;
