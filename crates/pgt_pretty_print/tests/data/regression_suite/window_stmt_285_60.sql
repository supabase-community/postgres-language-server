SELECT * FROM
  (SELECT empno,
          depname,
          salary,
          count(empno) OVER (PARTITION BY depname ORDER BY salary DESC) c
   FROM empsalary) emp
WHERE c <= 3;
