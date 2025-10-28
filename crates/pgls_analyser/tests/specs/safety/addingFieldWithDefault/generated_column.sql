-- expect_lint/safety/addingFieldWithDefault
ALTER TABLE users ADD COLUMN full_name text GENERATED ALWAYS AS (first_name || ' ' || last_name) STORED;
