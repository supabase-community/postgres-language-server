SELECT t1.c, count(t2.c) FROM pagg_tab3 t1 JOIN pagg_tab4 t2 ON t1.c = t2.c AND t1.c = t2.b GROUP BY 1 ORDER BY t1.c COLLATE "C";
