SELECT * FROM check_estimated_rows('SELECT * FROM expr_stats WHERE (a+b) = 0 AND (a-b) = 0');
