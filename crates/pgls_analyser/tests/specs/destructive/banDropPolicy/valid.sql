-- expect_no_diagnostics
create policy p on t using (true);
alter policy p on t using (true);
