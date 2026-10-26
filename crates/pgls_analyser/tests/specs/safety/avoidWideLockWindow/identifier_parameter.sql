-- expect_lint/avoidWideLockWindow
-- The name of the column is a psql variable, but the table is known, so its lock is held.
ALTER TABLE users ADD COLUMN :"column" TEXT;
ALTER TABLE orders ADD COLUMN total NUMERIC;
