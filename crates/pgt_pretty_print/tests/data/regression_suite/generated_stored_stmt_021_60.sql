CREATE TABLE gtest_err_7d (a int PRIMARY KEY, b int GENERATED ALWAYS AS (generate_series(1, a)) STORED);
