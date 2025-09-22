SELECT * FROM check_estimated_rows('SELECT * FROM expr_stats WHERE a = 0 AND b = 1 AND (a-b) = 0');
