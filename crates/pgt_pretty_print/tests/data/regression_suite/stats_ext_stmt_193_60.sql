SELECT * FROM check_estimated_rows('SELECT COUNT(*) FROM ndistinct GROUP BY (a+1), (b+100), (2*c)');
