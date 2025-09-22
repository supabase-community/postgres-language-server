do $$
declare cnt int := 0;
  c1 cursor for select * from forc_test;
begin
  for r1 in c1 loop
    declare c1 cursor for select * from forc_test;
    begin
      for r2 in c1 loop
        cnt := cnt + 1;
      end loop;
    end;
  end loop;
  raise notice 'cnt = %', cnt;
end $$;
