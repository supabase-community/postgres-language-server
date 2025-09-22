insert into rw_view1 values ('zzz',2.0,1)
  on conflict (aa) do update set bb = rw_view1.bb||'xxx';
