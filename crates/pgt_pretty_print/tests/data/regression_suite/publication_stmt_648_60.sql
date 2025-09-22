CREATE TABLE gencols (a int, gen1 int GENERATED ALWAYS AS (a * 2) STORED);
