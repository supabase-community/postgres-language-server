-- expect_lint/safety/multipleAlterTable
-- Test ALTER TABLE after other statements
CREATE TABLE products (id serial PRIMARY KEY);
ALTER TABLE products ADD COLUMN description text;
ALTER TABLE products ADD COLUMN price numeric;
