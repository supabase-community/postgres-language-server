CREATE VIEW rw_view1 AS
  SELECT *, 'Const' AS c, (SELECT concat('b: ', b)) AS d FROM base_tbl WHERE a>0;
