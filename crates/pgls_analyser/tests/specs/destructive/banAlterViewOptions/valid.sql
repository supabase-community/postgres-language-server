-- expect_no_diagnostics
alter view v rename to w;
alter table t set (fillfactor = 70);
