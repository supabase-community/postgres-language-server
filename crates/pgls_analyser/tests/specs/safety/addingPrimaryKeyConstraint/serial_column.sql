-- expect_lint/safety/addingPrimaryKeyConstraint
ALTER TABLE items ADD COLUMN id SERIAL PRIMARY KEY;