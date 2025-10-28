-- expect_only_lint/safety/lockTimeoutWarning
-- ALTER TABLE without lock timeout should trigger the rule
ALTER TABLE authors ADD COLUMN email TEXT;