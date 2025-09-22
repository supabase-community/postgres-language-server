CREATE VIEW rw_view1 AS
  SELECT *, 'Const1' AS c1 FROM base_tbl WHERE a>0 OFFSET 0;
