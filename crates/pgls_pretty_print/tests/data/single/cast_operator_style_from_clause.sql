-- pgls-format: castStyle=operator
SELECT * FROM CAST(1 + 2 AS int) AS a, ROWS FROM (CAST('x' AS text)) AS b;
