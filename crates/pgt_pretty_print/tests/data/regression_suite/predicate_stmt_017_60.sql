SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON EXISTS
        (SELECT 1 FROM pred_tab t3, pred_tab t4, pred_tab t5, pred_tab t6
         WHERE t1.a = t3.a AND t6.a IS NOT NULL);
