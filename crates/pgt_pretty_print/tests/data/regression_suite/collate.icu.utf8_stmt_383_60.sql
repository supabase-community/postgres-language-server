SELECT * FROM outer_text WHERE (f1, f2) NOT IN (SELECT * FROM inner_text);
