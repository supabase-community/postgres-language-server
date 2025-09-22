SELECT * FROM check_estimated_rows('SELECT * FROM ab1 WHERE (case a when 1 then true else false end) AND b=2');
