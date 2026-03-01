-- expect_no_diagnostics
SET idle_in_transaction_session_timeout = '30s';
ALTER TABLE users ADD COLUMN email TEXT;
