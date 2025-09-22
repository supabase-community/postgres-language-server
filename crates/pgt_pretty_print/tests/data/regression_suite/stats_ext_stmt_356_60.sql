SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE (a * 2) IN (2, 102) AND upper(b) IN (''1'', ''2'')');
