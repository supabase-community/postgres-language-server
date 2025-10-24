-- Valid: Lock timeout is set before ALTER TABLE
-- expect_no_diagnostics
SET LOCAL lock_timeout = '2s';
ALTER TABLE authors ADD COLUMN email TEXT;
