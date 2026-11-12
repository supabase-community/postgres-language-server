-- Before Postgres 11, every default rewrites the table.
-- catalog: builtin
-- server_version_num: 100000
-- expect_lint/addingFieldWithDefault
ALTER TABLE users ADD COLUMN status text DEFAULT 'active';
