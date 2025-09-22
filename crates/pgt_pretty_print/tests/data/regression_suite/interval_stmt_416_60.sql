SELECT i AS interval, date_trunc('hour', i)
    FROM INFINITE_INTERVAL_TBL
    WHERE NOT isfinite(i);
