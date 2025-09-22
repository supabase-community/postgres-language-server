SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists WHERE mod(a,20) < 1 AND mod(b::int,10) < 1');
