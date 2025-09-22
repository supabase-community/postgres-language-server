CREATE TABLE gtest30 (
    a int,
    b int GENERATED ALWAYS AS (a * 2) STORED
);
