SELECT * FROM check_estimated_rows('SELECT COUNT(*) FROM ndistinct GROUP BY (a*5), b');
