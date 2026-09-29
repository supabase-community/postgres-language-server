-- Valid: Creating a regular table (not an enum)
-- expect_no_diagnostics
CREATE TABLE users (
    id INT PRIMARY KEY,
    name TEXT NOT NULL
);
