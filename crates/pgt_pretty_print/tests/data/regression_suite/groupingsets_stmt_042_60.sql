select x, y
  from (select four as x, four as y from tenk1) as t
  group by grouping sets (x, y)
  having y is null
  order by 1, 2;
