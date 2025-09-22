SELECT i AS interval, date_trunc('ago', i)
    FROM INFINITE_INTERVAL_TBL
    WHERE NOT isfinite(i);
