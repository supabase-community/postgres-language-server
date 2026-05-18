-- expect_lint/safety/requireStatementTimeout
SET statement_timeout = '5s';
COMMIT;
ALTER TABLE users ADD COLUMN email TEXT;
