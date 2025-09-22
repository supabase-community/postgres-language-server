select x, y || 'y'
  from (select four as x, four as y from tenk1) as t
  group by grouping sets (x, y)
  order by 1, 2;
