CREATE VIEW v1 WITH (security_barrier = true) AS
  SELECT * FROM t1;
