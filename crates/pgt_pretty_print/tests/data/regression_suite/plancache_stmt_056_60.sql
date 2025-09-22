create function cachebug() returns void as $$
declare r int;
begin
  drop table if exists temptable cascade;
  create temp table temptable as select * from generate_series(1,3) as f1;
  create temp view vv as select * from temptable;
  for r in select * from vv loop
    raise notice '%', r;
  end loop;
end$$ language plpgsql;
