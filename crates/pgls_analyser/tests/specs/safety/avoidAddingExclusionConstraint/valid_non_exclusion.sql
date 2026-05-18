-- expect_no_diagnostics
alter table my_table add constraint my_check check (col > 0) not valid;
