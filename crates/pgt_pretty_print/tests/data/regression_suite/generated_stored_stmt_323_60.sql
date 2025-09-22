CREATE TABLE gtest29 (
    a int,
    b int GENERATED ALWAYS AS (a * 2) STORED
);
