-- expect_lint/safety/banCharField
ALTER TABLE users ADD COLUMN code character(10);
