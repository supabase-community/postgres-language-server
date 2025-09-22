ALTER TABLE gtest20a ADD COLUMN c float8 DEFAULT random() CHECK (b < 61);
