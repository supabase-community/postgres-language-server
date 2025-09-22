select (select max(unique1) filter (where sum(ten) > 0) from int8_tbl) from tenk1;
