SELECT t AS timestamp, i AS interval,
       eval(format('timestamp %L + interval %L', t, i)) AS plus,
       eval(format('timestamp %L - interval %L', t, i)) AS minus
FROM (VALUES (timestamp '-infinity'),
             (timestamp '1995-08-06 12:30:15'),
             (timestamp 'infinity')) AS t1(t),
     (VALUES (interval '-infinity'),
             (interval 'infinity')) AS t2(i);
