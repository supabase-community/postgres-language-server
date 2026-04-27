-- expect_no_diagnostics
ALTER TABLE orders ADD CONSTRAINT orders_check CHECK (total > 0) NOT VALID;
COMMIT;
ALTER TABLE orders VALIDATE CONSTRAINT orders_check;
