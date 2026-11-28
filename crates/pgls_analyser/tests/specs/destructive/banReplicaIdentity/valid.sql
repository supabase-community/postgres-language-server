-- expect_no_diagnostics
alter table t enable row level security;
alter publication p add table t;
create publication p for table t;
