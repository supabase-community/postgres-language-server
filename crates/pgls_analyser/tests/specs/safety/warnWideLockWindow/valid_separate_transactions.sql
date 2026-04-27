-- expect_no_diagnostics
ALTER TABLE users ADD COLUMN email TEXT;
COMMIT;
ALTER TABLE orders ADD COLUMN total NUMERIC;
