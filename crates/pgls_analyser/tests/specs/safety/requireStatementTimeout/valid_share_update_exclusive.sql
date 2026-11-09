-- expect_no_diagnostics
-- SHARE UPDATE EXCLUSIVE doesn't block reads or writes.
alter table orders validate constraint orders_user_fk;
alter table orders set (autovacuum_vacuum_scale_factor = 0.01);
alter table events detach partition events_2023 concurrently;
