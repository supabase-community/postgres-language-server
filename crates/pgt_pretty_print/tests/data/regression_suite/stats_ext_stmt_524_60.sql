SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists WHERE 1 > mod(a,20) AND 1 > mod(b::int,10)');
