CREATE VIEW rw_view2 AS
  SELECT aa AS aaa, bb AS bbb, c AS c1, 'Const2' AS c2 FROM rw_view1 WHERE aa<10;
