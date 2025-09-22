CREATE TABLE gtest1_y (b int GENERATED ALWAYS AS (x + 1) STORED) INHERITS (gtest1, gtesty);
