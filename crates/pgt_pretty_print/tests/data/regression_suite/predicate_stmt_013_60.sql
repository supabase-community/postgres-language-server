SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON t1.a IS NOT NULL OR t2.b = 1;
