ALTER TABLE gtestnn_parent ALTER COLUMN f3 SET EXPRESSION AS (nullif(f1, 2) + nullif(f2, 11));
