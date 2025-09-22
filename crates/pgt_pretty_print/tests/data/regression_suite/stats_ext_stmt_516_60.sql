SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists WHERE mod(a,20) < ALL (ARRAY[4, 5]) AND mod(b::int,10) IN (1, 2, 3) AND mod(c,5) > ANY (ARRAY[1, 2, 3])');
