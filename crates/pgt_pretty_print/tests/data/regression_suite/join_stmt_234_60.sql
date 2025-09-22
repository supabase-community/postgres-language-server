select a.* from tenk1 a
where not exists (select 1 from tenk1 b where a.unique1 = b.unique2);
