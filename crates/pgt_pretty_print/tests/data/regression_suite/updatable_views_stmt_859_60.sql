create view uv_iocu_view as
   select b, b+1 as c, a, '2.0'::text as two from uv_iocu_tab;
