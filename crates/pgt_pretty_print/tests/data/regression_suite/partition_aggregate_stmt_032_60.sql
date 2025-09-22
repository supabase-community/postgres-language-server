SELECT c, sum(b order by a) FROM pagg_tab GROUP BY c ORDER BY 1, 2;
