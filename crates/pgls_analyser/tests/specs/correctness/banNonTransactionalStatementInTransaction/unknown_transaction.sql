-- expect_no_diagnostics
-- Without section markers, we don't know whether the file runs in a transaction.
vacuum users;
vacuum posts;
