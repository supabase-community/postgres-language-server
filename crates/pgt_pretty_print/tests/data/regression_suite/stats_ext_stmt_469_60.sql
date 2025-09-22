SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists WHERE a = ANY (ARRAY[4,5]) AND 4 = ANY(ia)');
