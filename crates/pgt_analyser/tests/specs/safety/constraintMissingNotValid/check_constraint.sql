-- expect_lint/safety/constraintMissingNotValid
ALTER TABLE users ADD CONSTRAINT check_age CHECK (age >= 0);
