-- NOT NULL ... NOT VALID (Postgres 18) is validated like a CHECK constraint
-- expect_lint/requireSeparateConstraintValidation
ALTER TABLE orders ADD CONSTRAINT orders_total_not_null NOT NULL total NOT VALID;
ALTER TABLE orders VALIDATE CONSTRAINT orders_total_not_null;
