SELECT * FROM pred_tab t1
    FULL JOIN pred_tab t2 ON t1.a = t2.a
    LEFT JOIN pred_tab t3 ON t2.a IS NOT NULL;
