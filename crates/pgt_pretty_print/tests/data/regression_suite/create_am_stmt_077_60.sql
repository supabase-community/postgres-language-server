CREATE TABLE am_partitioned(x INT, y INT) PARTITION BY hash (x) USING heap2;
