-- expect_only_lint/safety/runningStatementWhileHoldingAccessExclusive
-- Running SELECT after ALTER TABLE should trigger the rule
ALTER TABLE authors ADD COLUMN email TEXT NOT NULL;
SELECT COUNT(*) FROM authors;