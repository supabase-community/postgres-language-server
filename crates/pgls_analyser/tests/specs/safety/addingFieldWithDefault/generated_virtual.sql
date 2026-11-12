-- A virtual generated column (Postgres 18) is computed on read, so there is no rewrite
-- expect_no_diagnostics
ALTER TABLE users ADD COLUMN full_name text GENERATED ALWAYS AS (first_name || ' ' || last_name) VIRTUAL;
ALTER TABLE users ADD COLUMN initials text GENERATED ALWAYS AS (left(first_name, 1) || left(last_name, 1));
