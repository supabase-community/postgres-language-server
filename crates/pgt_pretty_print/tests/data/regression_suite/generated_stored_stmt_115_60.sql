CREATE TABLE gtest3a (a text, b text GENERATED ALWAYS AS (a || '+' || a) STORED);
