SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE ((a * 2) = 2 OR upper(b) = ''1'') AND upper(b) = ''1''');
