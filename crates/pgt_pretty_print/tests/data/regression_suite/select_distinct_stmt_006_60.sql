SELECT count(*) FROM
  (SELECT DISTINCT two, four, two FROM tenk1) ss;
