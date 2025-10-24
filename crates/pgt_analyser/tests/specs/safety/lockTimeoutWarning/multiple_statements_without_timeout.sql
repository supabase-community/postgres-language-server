-- Both statements should trigger the rule
-- expect_lint/safety/lockTimeoutWarning
CREATE INDEX orders_user_idx ON orders(user_id);

-- expect_lint/safety/lockTimeoutWarning
ALTER TABLE orders ADD COLUMN total DECIMAL(10, 2);
