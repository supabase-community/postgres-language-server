CREATE TABLE gtest_err_7b (a int PRIMARY KEY, b int GENERATED ALWAYS AS (row_number() OVER (ORDER BY a)) STORED);
