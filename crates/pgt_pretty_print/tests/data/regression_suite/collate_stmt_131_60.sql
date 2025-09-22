CREATE VIEW collate_on_int AS
SELECT c1+1 AS c1p FROM
  (SELECT ('4' COLLATE "C")::INT AS c1) ss;
