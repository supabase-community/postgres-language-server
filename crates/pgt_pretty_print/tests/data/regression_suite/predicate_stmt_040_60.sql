SELECT * FROM pred_tab t1
    LEFT JOIN pred_tab t2 ON TRUE
    LEFT JOIN pred_tab_notnull t3 ON t2.a = t3.a
    LEFT JOIN pred_tab t4 ON t3.b IS NULL AND t3.a IS NOT NULL;
