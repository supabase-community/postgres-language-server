SELECT t AT TIME ZONE 'GMT' AS timestamptz, i AS interval,
       eval(format('timestamptz %L + interval %L', t, i)) AS plus,
       eval(format('timestamptz %L - interval %L', t, i)) AS minus
FROM (VALUES (timestamptz '-infinity'),
             (timestamptz '1995-08-06 12:30:15 GMT'),
             (timestamptz 'infinity')) AS t1(t),
     (VALUES (interval '-infinity'),
             (interval 'infinity')) AS t2(i);
