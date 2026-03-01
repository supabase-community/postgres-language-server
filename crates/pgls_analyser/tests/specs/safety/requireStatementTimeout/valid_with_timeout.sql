-- expect_no_diagnostics
SET statement_timeout = '5s';
ALTER TABLE users ADD COLUMN email TEXT;
