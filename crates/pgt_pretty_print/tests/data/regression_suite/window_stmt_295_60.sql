SELECT * FROM
  (SELECT empno,
          salary,
          count(random()) OVER (ORDER BY empno DESC) c
   FROM empsalary) emp
WHERE c = 1;
