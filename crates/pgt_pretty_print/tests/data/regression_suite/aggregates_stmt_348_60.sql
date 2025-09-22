select (select max(unique1) filter (where bool_or(ten > 0)) from int8_tbl) from tenk1;
