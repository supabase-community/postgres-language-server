CREATE TABLE gtest2 (a int, b text GENERATED ALWAYS AS (a || ' sec') STORED);
