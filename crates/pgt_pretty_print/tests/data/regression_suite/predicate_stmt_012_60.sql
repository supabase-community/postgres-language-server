SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON t1.a = 1
    LEFT JOIN pred_tab t3 ON t2.a IS NULL;
