SELECT i AS interval,
    -i AS um,
    i * 2.0 AS mul,
    i * -2.0 AS mul_neg,
    i * 'infinity' AS mul_inf,
    i * '-infinity' AS mul_inf_neg,
    i / 3.0 AS div,
    i / -3.0 AS div_neg
    FROM INFINITE_INTERVAL_TBL
    WHERE NOT isfinite(i);
