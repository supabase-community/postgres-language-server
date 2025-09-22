SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists WHERE mod(a,7) = 1 AND mod(b::int,11) = 1');
