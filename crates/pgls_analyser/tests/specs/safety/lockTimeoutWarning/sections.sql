-- section: up
SET LOCAL lock_timeout = '1s';
ALTER TABLE authors ADD COLUMN email text;

-- The down section runs in a session of its own.
-- section: down
-- expect_lint/lockTimeoutWarning
ALTER TABLE authors DROP COLUMN email;
