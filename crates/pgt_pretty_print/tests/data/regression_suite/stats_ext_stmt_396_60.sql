SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE (a * 2) = ANY (ARRAY[2, 4, 102, 104]) AND upper(b) = ANY (ARRAY[''1'', ''2''])');
