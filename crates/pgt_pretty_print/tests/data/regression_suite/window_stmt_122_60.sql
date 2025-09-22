SELECT sum(unique1) over (rows between x preceding and x following),
         unique1, four
  FROM tenk1 WHERE unique1 < 10;
