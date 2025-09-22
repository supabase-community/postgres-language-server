SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE (a * 2) = 2 AND upper(b) = ''1'' AND (c + 1) = 2');
