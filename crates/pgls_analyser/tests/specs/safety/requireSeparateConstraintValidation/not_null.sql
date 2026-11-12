-- A NOT NULL constraint (Postgres 18) added as NOT VALID too
-- expect_lint/requireSeparateConstraintValidation
ALTER TABLE orders ADD CONSTRAINT orders_total_not_null NOT NULL total NOT VALID;
ALTER TABLE orders VALIDATE CONSTRAINT orders_total_not_null;
