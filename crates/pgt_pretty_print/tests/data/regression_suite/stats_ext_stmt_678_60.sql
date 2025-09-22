SELECT * FROM check_estimated_rows('SELECT * FROM expr_stats WHERE a = 0 AND (b || c) <= ''z'' AND (c || b) >= ''0''');
