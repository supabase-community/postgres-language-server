-- expect_lint/safety/preferRobustStmts
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
SELECT 1;