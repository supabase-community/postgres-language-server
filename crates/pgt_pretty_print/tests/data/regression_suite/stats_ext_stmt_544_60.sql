SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists WHERE mod(a,20) <= ANY (ARRAY[1, NULL, 2, 3]) AND mod(b::int,10) IN (1, 2, NULL, 3)');
