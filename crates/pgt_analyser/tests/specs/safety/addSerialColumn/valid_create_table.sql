-- Test CREATE TABLE with serial column (should be safe, rule only applies to ALTER TABLE)
-- expect_no_diagnostics
CREATE TABLE products (
    id serial PRIMARY KEY,
    name text NOT NULL,
    price numeric
);
