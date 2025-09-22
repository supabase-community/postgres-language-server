SELECT x, avg(x) OVER(ROWS BETWEEN CURRENT ROW AND 2 FOLLOWING)
FROM (VALUES (NULL::interval),
               ('3 days'::interval),
               ('infinity'::timestamptz - now()),
               ('6 days'::interval),
               ('-infinity'::interval)) v(x);
