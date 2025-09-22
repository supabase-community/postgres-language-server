select a.* from tenk1 a
where unique1 not in (select unique2 from tenk1 b);
