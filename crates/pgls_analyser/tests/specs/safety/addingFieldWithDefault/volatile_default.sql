-- expect_lint/addingFieldWithDefault
ALTER TABLE users ADD COLUMN created_at timestamp DEFAULT now();
