-- expect_no_diagnostics
-- SHARE UPDATE EXCLUSIVE doesn't block reads or writes.
alter table orders validate constraint orders_user_fk;
