SELECT * FROM
  (SELECT empno,
          salary,
          count(*) OVER (ORDER BY salary DESC ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING) c
   FROM empsalary) emp
WHERE c >= 3;
