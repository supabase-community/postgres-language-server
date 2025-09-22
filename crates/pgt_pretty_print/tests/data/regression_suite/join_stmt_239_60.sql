select a.* from tenk1 a
where exists (select 1 from tenk1 b where a.unique1 = b.unique2 group by b.unique1);
