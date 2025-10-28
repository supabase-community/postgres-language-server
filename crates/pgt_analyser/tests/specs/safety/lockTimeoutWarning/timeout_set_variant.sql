-- Valid: SET lock_timeout (without LOCAL) also works
-- expect_no_diagnostics
SET lock_timeout = '1s';
ALTER TABLE books ADD COLUMN author_id INT;
