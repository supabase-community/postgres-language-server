SELECT c, sum(a) FROM pagg_tab GROUP BY rollup(c) ORDER BY 1, 2;
