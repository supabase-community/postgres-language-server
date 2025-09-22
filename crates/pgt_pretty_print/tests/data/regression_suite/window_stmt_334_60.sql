SELECT  x
        ,avg(x) OVER(ROWS BETWEEN CURRENT ROW AND 1 FOLLOWING ) as curr_next_avg
        ,avg(x) OVER(ROWS BETWEEN 1 PRECEDING AND CURRENT ROW ) as prev_curr_avg
        ,sum(x) OVER(ROWS BETWEEN CURRENT ROW AND 1 FOLLOWING ) as curr_next_sum
        ,sum(x) OVER(ROWS BETWEEN 1 PRECEDING AND CURRENT ROW ) as prev_curr_sum
FROM (VALUES (NULL::interval),
               ('infinity'::interval),
               ('-2147483648 days -2147483648 months -9223372036854775807 usecs'), -- extreme interval value
               ('-infinity'::interval),
               ('2147483647 days 2147483647 months 9223372036854775806 usecs'), -- extreme interval value
               ('infinity'::interval),
               ('6 days'::interval),
               ('7 days'::interval),
               (NULL::interval),
               ('-infinity'::interval)) v(x);
