select a.* from tenk1 a
where unique1 in (select unique2 from tenk1 b);
