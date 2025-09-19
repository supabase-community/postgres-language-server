-- expect_lint/safety/constraintMissingNotValid
ALTER TABLE distributors ADD CONSTRAINT distfk FOREIGN KEY (address) REFERENCES addresses (address);