SELECT sum((3 = ANY(SELECT lag(x) over(order by x)
                    FROM generate_series(1,4) x))::int);
