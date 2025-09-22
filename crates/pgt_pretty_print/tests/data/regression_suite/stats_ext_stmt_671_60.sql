SELECT * FROM check_estimated_rows('SELECT * FROM expr_stats WHERE a = 0 AND (2*a) = 0 AND (3*b) = 0');
