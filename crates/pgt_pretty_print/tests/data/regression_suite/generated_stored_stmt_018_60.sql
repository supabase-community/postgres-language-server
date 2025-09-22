CREATE TABLE gtest_err_7a (a int PRIMARY KEY, b int GENERATED ALWAYS AS (avg(a)) STORED);
