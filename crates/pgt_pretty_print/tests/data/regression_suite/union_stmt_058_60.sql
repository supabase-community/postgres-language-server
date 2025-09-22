select f1 from int4_tbl union all
  (select unique1 from tenk1 union select unique2 from tenk1);
