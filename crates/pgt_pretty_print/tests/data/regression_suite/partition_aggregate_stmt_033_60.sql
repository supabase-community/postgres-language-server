SELECT a, sum(b order by a) FROM pagg_tab GROUP BY a ORDER BY 1, 2;
