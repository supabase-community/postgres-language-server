-- expect_lint/safety/addingFieldWithDefault
ALTER TABLE users ADD COLUMN created_at timestamp DEFAULT now();
