select count(*) from
  ( select unique1 from tenk1 union select fivethous from tenk1 ) ss;
