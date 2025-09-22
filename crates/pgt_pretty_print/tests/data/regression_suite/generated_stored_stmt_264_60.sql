CREATE TABLE gtest_parent (f1 date NOT NULL, f2 bigint, f3 bigint) PARTITION BY RANGE (f1);
