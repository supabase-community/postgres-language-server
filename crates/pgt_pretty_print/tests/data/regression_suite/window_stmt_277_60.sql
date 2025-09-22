SELECT * FROM
  (SELECT empno,
          salary,
          count(empno) OVER (ORDER BY salary DESC) c
   FROM empsalary) emp
WHERE c <= 3;
