-- expect_lint/banReplaceViewFunction
create or replace rule r as on insert to t do instead nothing;
