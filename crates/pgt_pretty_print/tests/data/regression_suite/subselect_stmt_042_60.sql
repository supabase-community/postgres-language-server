CREATE VIEW view_unnamed_ss_locking AS
SELECT * FROM (SELECT * FROM int4_tbl), int8_tbl AS unnamed_subquery
  WHERE f1 = q1
  FOR UPDATE OF unnamed_subquery;
