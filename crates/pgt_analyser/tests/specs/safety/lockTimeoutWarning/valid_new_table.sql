-- Valid: ALTER TABLE on a table created in the same transaction doesn't need timeout
-- expect_no_diagnostics
CREATE TABLE users (
    id INT PRIMARY KEY,
    name TEXT
);

ALTER TABLE users ADD COLUMN email TEXT;
