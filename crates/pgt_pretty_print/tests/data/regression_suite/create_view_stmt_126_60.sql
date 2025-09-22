create view aliased_order_by as
select x1 as x2, x2 as x1, x3 from tt1
  order by x2;
