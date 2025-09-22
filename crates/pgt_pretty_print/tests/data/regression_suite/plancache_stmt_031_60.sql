create or replace temp view v1 as
  select 2+2+4+(select max(unique1) from tenk1) as f1;
