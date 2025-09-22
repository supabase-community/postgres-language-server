SELECT * FROM
  (SELECT row_number() OVER (PARTITION BY salary) AS rn,
          lead(depname) OVER (PARTITION BY salary) || ' Department' AS n_dep
   FROM empsalary) emp
WHERE rn < 1;
