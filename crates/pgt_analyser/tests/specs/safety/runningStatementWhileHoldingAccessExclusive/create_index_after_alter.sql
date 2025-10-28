-- expect_only_lint/safety/runningStatementWhileHoldingAccessExclusive
-- CREATE INDEX after ALTER TABLE should trigger
ALTER TABLE orders ADD COLUMN total DECIMAL(10, 2);
CREATE INDEX orders_total_idx ON orders(total);
