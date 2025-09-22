SELECT
    empno,
    depname,
    row_number() OVER (PARTITION BY depname ORDER BY enroll_date) rn,
    rank() OVER (PARTITION BY depname ORDER BY enroll_date ROWS BETWEEN
                 UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) rnk,
    dense_rank() OVER (PARTITION BY depname ORDER BY enroll_date RANGE BETWEEN
                       CURRENT ROW AND CURRENT ROW) drnk,
    ntile(10) OVER (PARTITION BY depname ORDER BY enroll_date RANGE BETWEEN
                    CURRENT ROW AND UNBOUNDED FOLLOWING) nt,
    percent_rank() OVER (PARTITION BY depname ORDER BY enroll_date ROWS BETWEEN
                         CURRENT ROW AND UNBOUNDED FOLLOWING) pr,
    cume_dist() OVER (PARTITION BY depname ORDER BY enroll_date RANGE BETWEEN
                      CURRENT ROW AND UNBOUNDED FOLLOWING) cd
FROM empsalary;
