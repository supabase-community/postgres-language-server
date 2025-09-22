CREATE VIEW view_unnamed_ss AS
SELECT * FROM (SELECT * FROM (SELECT abs(f1) AS a1 FROM int4_tbl)),
              (SELECT * FROM int8_tbl)
  WHERE a1 < 10 AND q1 > a1 ORDER BY q1, q2;
