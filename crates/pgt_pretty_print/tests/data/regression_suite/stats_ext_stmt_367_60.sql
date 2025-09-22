SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE (a * 2) = ANY (ARRAY[2, 102]) AND upper(b) = ANY (ARRAY[''1'', ''2''])');
