-- expect_lint/safety/warnWideLockWindow
ALTER TABLE users ADD COLUMN email TEXT;
ALTER TABLE orders ADD COLUMN total NUMERIC;
