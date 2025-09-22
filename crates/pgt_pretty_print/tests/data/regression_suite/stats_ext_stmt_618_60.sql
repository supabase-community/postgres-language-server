SELECT * FROM check_estimated_rows('SELECT * FROM mcv_lists_partial WHERE (a = 0 AND b = 0 AND c = 0) OR (a = 1 AND b = 1 AND c = 1) OR (a = 2 AND b = 2 AND c = 2)');
