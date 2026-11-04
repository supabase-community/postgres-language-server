-- dbmate sends a section to Postgres as one query, which runs in a transaction block if it has
-- more than one statement, so the lock is held until the end of the section.
-- section: up transaction:false
ALTER TABLE books ADD COLUMN isbn text;
-- expect_lint/runningStatementWhileHoldingAccessExclusive
UPDATE books SET isbn = '';
