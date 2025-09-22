SELECT * FROM check_estimated_rows('SELECT * FROM functional_dependencies WHERE (a * 2) = ANY (ARRAY[2, 52, 102, 152]) AND upper(b) = ANY (ARRAY[''1'', ''26'']) AND (c + 1) = 2');
