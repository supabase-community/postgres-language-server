SELECT i AS interval, date_trunc('week', i)
    FROM INFINITE_INTERVAL_TBL
    WHERE NOT isfinite(i);
