SELECT * FROM
  (SELECT empno,
          salary,
          count((SELECT 1)) OVER (ORDER BY empno DESC) c
   FROM empsalary) emp
WHERE c = 1;
