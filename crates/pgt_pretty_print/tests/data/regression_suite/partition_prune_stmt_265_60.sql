select * from
  (select exists (select 1 from int4_tbl tinner where f1 = touter.f1) as b
   from int4_tbl touter) ss,
  asptab
where asptab.id > ss.b::int;
