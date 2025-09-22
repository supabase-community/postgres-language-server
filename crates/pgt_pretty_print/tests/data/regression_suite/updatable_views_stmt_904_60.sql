insert into rw_view1 values ('zzz',2.0,1)
  on conflict (aa) do update set cc = 3.0;
