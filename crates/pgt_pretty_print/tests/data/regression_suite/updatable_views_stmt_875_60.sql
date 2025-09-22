insert into uv_iocu_view (aa,bb) values (1,'y')
   on conflict (aa) do update set bb = 'Rejected: '||excluded.*
   where excluded.aa > 0
   and excluded.bb != ''
   and excluded.cc is not null;
