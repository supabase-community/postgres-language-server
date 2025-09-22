SELECT i AS interval, justify_days(i), justify_hours(i), justify_interval(i)
    FROM INFINITE_INTERVAL_TBL
    WHERE NOT isfinite(i);
