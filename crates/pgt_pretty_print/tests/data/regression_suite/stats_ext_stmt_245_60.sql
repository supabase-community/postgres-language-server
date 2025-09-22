SELECT * FROM check_estimated_rows('SELECT COUNT(*) FROM ndistinct GROUP BY a, (b+1), c, (d - 1)');
