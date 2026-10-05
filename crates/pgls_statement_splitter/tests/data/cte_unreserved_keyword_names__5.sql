WITH constraints AS (SELECT 1 AS n),
     indexes AS (SELECT 2 AS n),
     comments AS (SELECT 3 AS n),
     functions AS (SELECT 4 AS n)
SELECT * FROM constraints, indexes, comments, functions;

WITH name AS (SELECT 1 AS n), source AS (SELECT 2 AS n), data AS (SELECT 3 AS n)
SELECT * FROM name, source, data;

WITH RECURSIVE nodes(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM nodes WHERE n < 3),
     version AS MATERIALIZED (SELECT 1 AS v)
SELECT * FROM nodes, version;

WITH int AS (SELECT 1 AS n), values AS (SELECT 2 AS n)
SELECT * FROM int, values;

WITH recursive AS (SELECT 1 AS n)
SELECT * FROM recursive;
