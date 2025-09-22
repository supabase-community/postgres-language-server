select a.* from tenk1 a left join tenk1 b on a.unique1 = b.unique2
where b.unique2 is null;
