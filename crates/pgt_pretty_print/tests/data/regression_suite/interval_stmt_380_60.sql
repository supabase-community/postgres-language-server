SELECT i1 AS interval1, i2 AS interval2,
       eval(format('interval %L + interval %L', i1, i2)) AS plus,
       eval(format('interval %L - interval %L', i1, i2)) AS minus
FROM (VALUES (interval '-infinity'),
             (interval '2 months'),
             (interval 'infinity')) AS t1(i1),
     (VALUES (interval '-infinity'),
             (interval '10 days'),
             (interval 'infinity')) AS t2(i2);
