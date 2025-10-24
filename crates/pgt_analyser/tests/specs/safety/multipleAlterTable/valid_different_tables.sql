-- Test ALTER TABLE on different tables (should be safe)
-- expect_no_diagnostics
ALTER TABLE authors ADD COLUMN bio text;
ALTER TABLE posts ADD COLUMN published_at timestamp;
