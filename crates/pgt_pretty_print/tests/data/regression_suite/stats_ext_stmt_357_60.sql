SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE (a * 2) IN (2, 4, 102, 104) AND upper(b) IN (''1'', ''2'')');
