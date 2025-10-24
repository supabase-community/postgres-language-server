-- Valid: CREATE INDEX CONCURRENTLY doesn't take dangerous locks
-- expect_no_diagnostics
CREATE INDEX CONCURRENTLY books_title_idx ON books(title);
