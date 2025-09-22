select tname, attname from (
select relname::information_schema.sql_identifier as tname, * from
  (select * from pg_class c) ss1) ss2
  right join pg_attribute a on a.attrelid = ss2.oid
where tname = 'tenk1' and attnum = 1;
