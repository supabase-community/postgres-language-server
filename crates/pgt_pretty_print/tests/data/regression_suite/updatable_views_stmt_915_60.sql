insert into rw_view3 (aa,bb) values (1,'xxx')
  on conflict (aa) do update set bb = excluded.bb;
