SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists WHERE mod(a,20) = 1 OR mod(b::int,10) = 1 OR mod(c,25) = 1 OR d IS NOT NULL');
