CREATE VIEW v2 WITH (security_barrier = true) AS
  SELECT * FROM v1 WHERE EXISTS (SELECT 1);
