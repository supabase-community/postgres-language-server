-- expect_lint/safety/banNotValidValidateSameTransaction
ALTER TABLE orders ADD CONSTRAINT orders_check CHECK (total > 0) NOT VALID;
ALTER TABLE orders VALIDATE CONSTRAINT orders_check;
