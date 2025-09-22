SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE (a * 2) IN (2, 52, 102, 152) AND upper(b) IN (''1'', ''26'') AND (c + 1) IN (2)');
