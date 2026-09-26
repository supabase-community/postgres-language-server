-- pgls-format: layout=expanded, indentStyle=tabs, indentSize=4
SELECT t.a, t.b FROM s.t WHERE t.c = 1 GROUP BY t.a ORDER BY t.a;
