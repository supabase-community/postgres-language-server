SELECT * FROM
  (SELECT empno,
          salary,
          count(*) OVER () c
   FROM empsalary) emp
WHERE 11 <= c;
