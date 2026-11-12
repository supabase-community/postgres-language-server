-- ATTACH PARTITION locks the partition ACCESS EXCLUSIVE.
alter table events attach partition events_2024 for values from ('2024-01-01') to ('2025-01-01');
-- expect_lint/runningStatementWhileHoldingAccessExclusive
select count(*) from events_2024;
