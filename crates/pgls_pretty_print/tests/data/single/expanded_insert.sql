-- pgls-format: layout=expanded, indentStyle=tabs, indentSize=4
INSERT INTO t.x (a, b) WITH c AS (SELECT 1 AS a, 2 AS b FROM t.y) SELECT c.a, c.b FROM c;
