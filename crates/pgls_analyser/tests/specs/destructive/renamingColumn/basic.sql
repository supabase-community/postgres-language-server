-- expect_lint/renamingColumn
ALTER TABLE users RENAME COLUMN name TO full_name;
-- expect_lint/renamingColumn
alter type address rename attribute zip to postal_code;
