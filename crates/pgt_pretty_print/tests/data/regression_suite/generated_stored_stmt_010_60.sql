CREATE TABLE gtest_err_2c (a int PRIMARY KEY,
    b int GENERATED ALWAYS AS (num_nulls(gtest_err_2c)) STORED);
