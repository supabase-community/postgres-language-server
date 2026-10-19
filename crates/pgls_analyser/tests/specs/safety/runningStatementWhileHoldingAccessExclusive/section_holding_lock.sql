-- section: up
ALTER TABLE authors ADD COLUMN email text;
-- expect_lint/runningStatementWhileHoldingAccessExclusive
UPDATE authors SET email = '';

-- section: down
ALTER TABLE authors DROP COLUMN email;
