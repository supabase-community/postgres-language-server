SELECT * FROM
  (SELECT empno,
          depname,
          salary,
          count(empno) OVER () c
   FROM empsalary) emp
WHERE c = 1;
