SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists WHERE mod(a,20) = ANY (ARRAY[1, 2, 51, 52]) AND mod(b::int,10) = ANY (ARRAY[1, 2])');
