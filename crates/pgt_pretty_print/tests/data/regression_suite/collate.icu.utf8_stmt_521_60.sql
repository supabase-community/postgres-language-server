SELECT t1.c COLLATE "C", count(t2.c) FROM pagg_tab3 t1 JOIN pagg_tab3 t2 ON t1.c = t2.c COLLATE "C" GROUP BY t1.c COLLATE "C" ORDER BY t1.c COLLATE "C";
