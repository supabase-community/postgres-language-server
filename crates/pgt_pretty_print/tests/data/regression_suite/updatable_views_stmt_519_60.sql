CREATE TABLE base_tbl (id int, idplus1 int GENERATED ALWAYS AS (id + 1) STORED);
