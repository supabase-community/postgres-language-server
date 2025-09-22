SELECT * FROM
  (SELECT *,
          count(salary) OVER (PARTITION BY depname || '') c1, -- w1
          row_number() OVER (PARTITION BY depname) rn, -- w2
          count(*) OVER (PARTITION BY depname) c2, -- w2
          count(*) OVER (PARTITION BY '' || depname) c3, -- w3
          ntile(2) OVER (PARTITION BY depname) nt -- w2
   FROM empsalary
) e WHERE rn <= 1 AND c1 <= 3 AND nt < 2;
