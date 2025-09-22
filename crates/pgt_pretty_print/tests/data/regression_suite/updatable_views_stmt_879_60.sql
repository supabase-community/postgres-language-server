insert into uv_iocu_view (aa) values (1)
   on conflict (aa) do update set bb = 'Rejected: '||excluded.*;
