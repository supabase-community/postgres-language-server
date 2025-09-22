SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists WHERE mod(a,20) IN (1, 2, 51, 52, NULL) AND mod(b::int,10) IN ( 1, 2, NULL)');
