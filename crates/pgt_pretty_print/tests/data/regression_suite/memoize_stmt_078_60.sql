SELECT * FROM tab_anti t1 WHERE t1.a IN
 (SELECT a FROM tab_anti t2 WHERE t2.b IN
  (SELECT t1.b FROM tab_anti t3 WHERE t2.a > 1 OFFSET 0));
