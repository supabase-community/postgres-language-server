CREATE TABLE parted_col_comment (a int, b text) PARTITION BY LIST (a) WITH (fillfactor=100);
