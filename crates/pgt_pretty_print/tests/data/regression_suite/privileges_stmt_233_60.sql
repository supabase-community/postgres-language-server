CREATE VIEW atest12sbv WITH (security_barrier=true) AS
  SELECT * FROM atest12 WHERE b <<< 5;
