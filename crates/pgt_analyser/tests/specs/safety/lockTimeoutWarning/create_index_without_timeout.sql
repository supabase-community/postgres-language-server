-- expect_only_lint/safety/lockTimeoutWarning
-- CREATE INDEX without CONCURRENTLY or lock timeout should trigger the rule
CREATE INDEX books_title_idx ON books(title);
