insert into uv_iocu_view (a, b) values ('xyxyxy', 1)
   on conflict (a) do update set b = uv_iocu_view.b;
