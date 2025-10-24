-- Valid: Statements before ALTER TABLE are fine
-- expect_no_diagnostics
SELECT COUNT(*) FROM authors;
CREATE INDEX authors_name_idx ON authors(name);
