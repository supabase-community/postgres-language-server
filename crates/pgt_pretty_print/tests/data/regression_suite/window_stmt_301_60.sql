SELECT DISTINCT
       empno,
       enroll_date,
       depname,
       sum(salary) OVER (PARTITION BY depname order by empno) depsalary,
       min(salary) OVER (PARTITION BY depname order by enroll_date) depminsalary
FROM empsalary
ORDER BY depname, enroll_date;
