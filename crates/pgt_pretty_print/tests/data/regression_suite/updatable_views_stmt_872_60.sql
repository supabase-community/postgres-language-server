create view uv_iocu_view as
    select b as bb, a as aa, uv_iocu_tab::text as cc from uv_iocu_tab;
