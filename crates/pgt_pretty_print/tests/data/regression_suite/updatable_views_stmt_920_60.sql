insert into rw_view4 (aa,bb) values (1,'yyy')
  on conflict (aa) do update set bb = excluded.bb;
