SELECT * FROM check_estimated_rows('SELECT * FROM expr_stats WHERE a = 3 AND b = 3 AND (a-b) = 0');
