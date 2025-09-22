CREATE TABLE gtest_err_6a (a int PRIMARY KEY, b bool GENERATED ALWAYS AS (xmin <> 37) STORED);
