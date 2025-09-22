insert into rw_view2 (aa,bb) values (1,'xxx')
  on conflict (aa) do update set bb = excluded.bb;
