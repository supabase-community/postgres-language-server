-- expect_no_diagnostics
alter policy p on t using (true);
alter policy p on t rename to q;
