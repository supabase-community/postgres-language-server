SELECT d AS date, i AS interval,
       eval(format('date %L + interval %L', d, i)) AS plus,
       eval(format('date %L - interval %L', d, i)) AS minus
FROM (VALUES (date '-infinity'),
             (date '1995-08-06'),
             (date 'infinity')) AS t1(d),
     (VALUES (interval '-infinity'),
             (interval 'infinity')) AS t2(i);
