SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE ((a * 2) = 2 OR (a * 2) = 4 OR (a * 2) = 102 OR (a * 2) = 104) AND (upper(b) = ''1'' OR upper(b) = ''2'')');
