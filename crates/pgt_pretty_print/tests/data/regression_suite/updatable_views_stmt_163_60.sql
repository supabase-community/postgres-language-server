CREATE VIEW rw_view2 AS
  SELECT *, 'Const2' AS c2 FROM rw_view1 WHERE a<10;
