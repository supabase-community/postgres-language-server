-- All statements after ALTER TABLE should trigger
ALTER TABLE users ADD COLUMN age INT;

-- expect_lint/safety/runningStatementWhileHoldingAccessExclusive
UPDATE users SET age = 30;

-- expect_lint/safety/runningStatementWhileHoldingAccessExclusive
SELECT * FROM users;
