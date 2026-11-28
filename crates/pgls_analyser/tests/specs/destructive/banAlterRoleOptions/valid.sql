-- expect_no_diagnostics
alter role r rename to s;
alter user app rename to app2;
alter group g add user app;
