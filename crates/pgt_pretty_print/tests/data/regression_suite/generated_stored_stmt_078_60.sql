CREATE TABLE gtest_normal_child (a int, b int GENERATED ALWAYS AS (a * 2) STORED) INHERITS (gtest_normal);
