SELECT * FROM
  (SELECT empno,
          depname,
          row_number() OVER (PARTITION BY depname ORDER BY empno) rn
   FROM empsalary) emp
WHERE rn < 3;
