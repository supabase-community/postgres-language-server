-- expect_only_lint/safety/runningStatementWhileHoldingAccessExclusive
-- INSERT after ALTER TABLE should trigger
ALTER TABLE books ADD COLUMN isbn TEXT;
INSERT INTO books (title, isbn) VALUES ('Database Systems', '978-0-1234567-8-9');
