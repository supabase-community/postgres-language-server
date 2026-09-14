-- pgls-format: layout=expanded, indentStyle=tabs, indentSize=4
SELECT a FROM a LEFT JOIN b ON b.a = a.a JOIN c ON c.a = a.a;
