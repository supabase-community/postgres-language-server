-- expect_no_diagnostics
alter function f() rename to g;
alter procedure p() rename to q;
alter routine f() set schema private;
alter procedure p() owner to app;
