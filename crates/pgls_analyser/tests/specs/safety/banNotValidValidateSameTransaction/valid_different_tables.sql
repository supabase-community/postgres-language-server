-- expect_no_diagnostics
ALTER TABLE users ADD CONSTRAINT check_positive CHECK (amount > 0) NOT VALID;
ALTER TABLE orders VALIDATE CONSTRAINT check_positive;
