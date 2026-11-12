-- expect_no_diagnostics
-- The table is created in the same transaction, even though the name of a column is a psql
-- variable.
CREATE TABLE users (
    id INT PRIMARY KEY,
    :"column" TEXT
);

ALTER TABLE users ADD COLUMN email TEXT;
