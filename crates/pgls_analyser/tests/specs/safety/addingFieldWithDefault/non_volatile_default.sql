-- Test non-volatile default values (should be safe in PG 11+, but we are passing no PG version info in the tests)
-- expect_lint/safety/addingFieldWithDefault
ALTER TABLE users ADD COLUMN status text DEFAULT 'active';
